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
        store::Store,
        RecipeUtilities, Result, Value,
    },
    futures::stream::{self, StreamExt, TryStreamExt},
    serde::Serialize,
    sqlparser::ast::{OrderByExpr, Select},
    std::{cmp::Ordering, fmt::Debug},
    thiserror::Error as ThisError,
};

#[derive(ThisError, Serialize, Debug, PartialEq)]
pub enum SelectError {
    #[error("aggregate groups not supported")]
    GrouperMayNotContainAggregate,

    #[error("an aggregate was probably used where not allowed")]
    FinalSolveFailure,

    #[error("this should be impossible, please report")]
    UnreachableFinalSolveFailure,
    #[error("this should be impossible, please report")]
    Unreachable,
}

pub async fn select<'a, Key: 'static + Debug>(
    storage: &'a dyn Store<Key>,
    query: Select,
    order_by: Vec<OrderByExpr>,
) -> Result<LabelsAndRows> {
    let Plan {
        joins,
        select_items,
        constraint,
        groups,
        order_by,
        labels,
    } = Plan::new(storage, query, order_by).await?;

    let rows = stream::iter(joins)
        .map(Ok)
        .try_fold(vec![], |rows, join| async {
            join.execute(storage, rows).await
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
                    .collect::<Result<Vec<PlannedRecipe>>>(),
            ),
            Ok(false) => None,
            Err(error) => Some(Err(error)),
        })
        .collect::<Result<Vec<Vec<PlannedRecipe>>>>()?;

    let contains_aggregates = select_items
        .iter()
        .any(|select_item| !select_item.aggregates.is_empty());
    let contains_non_aggregates = select_items
        .iter()
        .any(|select_item| select_item.aggregates.is_empty());

    let final_rows = if contains_aggregates {
        if contains_non_aggregates {
            unimplemented!();
        } else {
            let mut ungrouped_groupers = selected_rows
                .into_iter()
                .zip(rows)
                .map(|(selected_row, row)| {
                    Ok((
                        groups
                            .iter()
                            .map(|group| {
                                group.clone().simplify_by_row(&row)?.confirm_or_err(
                                    SelectError::GrouperMayNotContainAggregate.into(),
                                )
                            })
                            .collect::<Result<Vec<Value>>>()?,
                        selected_row,
                    ))
                })
                .collect::<Result<Vec<(Vec<Value>, Vec<PlannedRecipe>)>>>()?;

            // I was originally thinking of doing this by sorting. That might still be a better method.
            let mut groups = vec![];
            while !ungrouped_groupers.is_empty() {
                let partitioner = ungrouped_groupers
                    .get(0)
                    .ok_or(SelectError::Unreachable)?
                    .0
                    .clone();
                let (partition, todo) = ungrouped_groupers
                    .into_iter()
                    .partition(|(groupers, _row)| groupers == &partitioner);
                ungrouped_groupers = todo;
                let partition = partition
                    .into_iter()
                    .map(|(_, selection)| selection)
                    .collect();
                groups.push(partition);
            }

            groups
                .into_iter()
                .map(|group: Vec<Vec<PlannedRecipe>>| {
                    let selections = group.get(0).ok_or(SelectError::Unreachable)?.clone();
                    let accumulated =
                        group
                            .into_iter()
                            .try_fold(vec![], |accumulators, selection| {
                                selection
                                    .into_iter()
                                    .zip(accumulators)
                                    .map(|(recipe, accumulators)| recipe.aggregate(accumulators))
                                    .collect::<Result<Vec<Row>>>() // TODO: Don't collect until end, fold into iter.
                            })?;

                    selections
                        .into_iter()
                        .zip(accumulated)
                        .map(|(selection, accumulated)| selection.solve_by_aggregate(accumulated))
                        .collect::<Result<Row>>()
                })
                .collect::<Result<Vec<Row>>>()?
        }
    } else {
        selected_rows
            .into_iter()
            .map(|selection| {
                selection
                    .into_iter()
                    .map(|selected| selected.confirm())
                    .collect::<Result<Row>>()
            })
            .collect::<Result<Vec<Row>>>()?
    };

    Ok((labels, final_rows))
}
