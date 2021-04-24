pub mod join;
mod manual;
mod order;
mod plan;

pub use {
    manual::{Manual, ManualError, SelectItem},
    order::Order,
    plan::{Plan, PlanError},
};

use {
    crate::{
        executor::{
            types::{LabelsAndRows, Row},
            PlannedRecipe,
        },
        macros::try_option,
        NullOrd, RecipeUtilities, Result, StorageInner, Value,
    },
    futures::stream::{self, StreamExt, TryStreamExt},
    rayon::prelude::*,
    serde::Serialize,
    sqlparser::ast::{OrderByExpr, Select},
    std::cmp::Ordering,
    thiserror::Error as ThisError,
};

#[derive(ThisError, Serialize, Debug, PartialEq)]
pub enum SelectError {
    #[error("aggregate groups not supported")]
    GrouperMayNotContainAggregate,

    #[error("an aggregate was probably used where not allowed")]
    FinalSolveFailure,

    #[error("HAVING does not yet support aggregates")]
    UnimplementedAggregateHaving,

    #[error("this should be impossible, please report")]
    UnreachableFinalSolveFailure,
    #[error("this should be impossible, please report")]
    Unreachable,
}

pub async fn select(
    storages: &Vec<(String, &mut StorageInner)>,
    query: Select,
    order_by: Vec<OrderByExpr>,
) -> Result<LabelsAndRows> {
    let Plan {
        joins,
        select_items,
        constraint,
        group_constraint,
        groups,
        order_by,
        labels,
    } = Plan::new(storages, query, order_by).await?;

    let rows = stream::iter(joins)
        .map(Ok)
        .try_fold(vec![], |rows, join| async {
            join.execute(storages, rows).await
        })
        .await?;

    let rows = order_by.execute(rows)?; // TODO: This should be done after filtering

    let selected_rows = rows
        .iter()
        .filter_map(|row| match constraint.confirm_constraint(row) {
            Ok(true) => Some(
                select_items
                    .iter()
                    .map(|selection| selection.clone().simplify_by_row(row))
                    .collect::<Result<Vec<PlannedRecipe>>>()
                    .map(|selection| (selection, row.clone())),
            ),
            Ok(false) => None,
            Err(error) => Some(Err(error)),
        })
        .collect::<Result<Vec<(Vec<PlannedRecipe>, Row)>>>()?;
    let do_group = !groups.is_empty()
        || select_items
            .iter()
            .any(|select_item| !select_item.aggregates.is_empty());

    let final_rows = if do_group {
        let groups = if groups.is_empty() {
            vec![PlannedRecipe::TRUE]
        } else {
            groups
        };
        let mut ungrouped_groupers = selected_rows
            .into_par_iter()
            .filter_map(|(selected_row, row)| {
                let group_constraint = try_option!(group_constraint.clone().simplify_by_row(&row));
                let group_constraint = match group_constraint.as_solution() {
                    Some(Value::Bool(true)) => None,
                    Some(Value::Bool(false)) => return None,
                    Some(_) => unreachable!(), // TODO: Handle
                    None => Some(group_constraint),
                };
                let groupers = try_option!(groups
                    .iter()
                    .map(|group| {
                        group
                            .clone()
                            .simplify_by_row(&row)?
                            .confirm_or_err(SelectError::GrouperMayNotContainAggregate.into())
                    })
                    .collect::<Result<Vec<Value>>>());
                Some(Ok((group_constraint, groupers, selected_row)))
            })
            .collect::<Result<Vec<(Option<PlannedRecipe>, Vec<Value>, Vec<PlannedRecipe>)>>>()?;

        ungrouped_groupers.sort_unstable_by(|groupers_a, groupers_b| {
            groupers_a
                .1
                .iter()
                .zip(&groupers_b.1)
                .find_map(|(grouper_a, grouper_b)| {
                    match grouper_a.null_cmp(grouper_b).unwrap_or(Ordering::Equal) {
                        Ordering::Equal => None,
                        other => Some(other),
                    }
                })
                .unwrap_or(Ordering::Equal)
        });
        let groups = ungrouped_groupers.into_iter().fold(
            vec![],
            |mut groups: Vec<(Vec<Value>, Vec<(Option<PlannedRecipe>, Vec<PlannedRecipe>)>)>,
             grouper: (Option<PlannedRecipe>, Vec<Value>, Vec<PlannedRecipe>)| {
                let value = &grouper.1;
                if let Some(last) = groups.last_mut() {
                    if &last.0 == value {
                        last.1.push((grouper.0, grouper.2));
                    } else {
                        groups.push((value.clone(), vec![(grouper.0, grouper.2)]));
                    }
                    groups
                } else {
                    vec![(value.clone(), vec![(grouper.0, grouper.2)])]
                }
            },
        );
        let groups: Vec<Vec<(Option<PlannedRecipe>, Vec<PlannedRecipe>)>> =
            groups.into_iter().map(|(_, group)| group).collect();

        groups
            .into_par_iter()
            .filter_map(|group: Vec<(Option<PlannedRecipe>, Vec<PlannedRecipe>)>| {
                // TODO: Improve
                let first_row =
                    try_option!(group.get(0).ok_or(SelectError::Unreachable.into())).clone();
                let group_constraint = first_row.0;
                if group_constraint.is_some() {
                    let group_constraint_accumulated = try_option!(group
                        .clone()
                        .into_iter()
                        .try_fold(vec![], |accumulators, (group_constraint, _)| {
                            group_constraint.unwrap().aggregate(accumulators)
                        }));
                    if matches!(
                        try_option!(group_constraint
                            .unwrap()
                            .solve_by_aggregate(group_constraint_accumulated)),
                        Value::Bool(false)
                    ) {
                        return None;
                    }
                }

                let selections = first_row.1;
                let accumulator_size = selections.len();
                let initial_accumulator = vec![vec![]; accumulator_size];
                let accumulated = try_option!(group.into_iter().try_fold(
                    initial_accumulator,
                    |accumulators, selection| {
                        selection
                            .1
                            .into_iter()
                            .zip(accumulators)
                            .map(|(recipe, accumulators)| recipe.aggregate(accumulators))
                            .collect::<Result<Vec<Row>>>()
                    },
                ));
                let result = selections
                    .clone()
                    .into_iter()
                    .zip(accumulated)
                    .map(|(selection, accumulated)| selection.solve_by_aggregate(accumulated))
                    .collect::<Result<Row>>();
                Some(result)
            })
            .collect::<Result<Vec<Row>>>()?
    } else {
        selected_rows
            .into_iter()
            .map(|(selection, _)| {
                selection
                    .into_iter()
                    .map(|selected| selected.confirm())
                    .collect::<Result<Row>>()
            })
            .collect::<Result<Vec<Row>>>()?
    };

    Ok((labels, final_rows))
}
