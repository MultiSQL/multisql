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
    std::fmt::Debug,
    thiserror::Error as ThisError,
};

macro_rules! try_option {
    ($try: expr) => {
        match $try {
            Ok(success) => success,
            Err(error) => return Some(Err(error)),
        }
    };
}

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

pub async fn select<'a, Key: 'static + Debug>(
    storage: &'a dyn Store<Key>,
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

    let do_group = !groups.is_empty()
        || select_items
            .iter()
            .any(|select_item| !select_item.aggregates.is_empty());

    //println!("Select debug:\nContains aggregates: {}\nContains non-aggregates: {}\nSelected Rows:\n\t{:?}", contains_aggregates, contains_non_aggregates, selected_rows);

    let final_rows = if do_group {
        /*if contains_non_aggregates {
            select_items
                .iter()
                .filter_map(|select_item| {
                    if select_item.aggregates.is_empty() {
                        Some()
                    } else {
                        None
                    }
                }).collect::<Result<Vec<usize>>>();
        }*/
        let groups = if groups.is_empty() {
            vec![PlannedRecipe::TRUE]
        } else {
            groups
        };
        let mut ungrouped_groupers = selected_rows
            .into_iter()
            .zip(rows)
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

        // I was originally thinking of doing this by sorting. That might still be a better method.
        let mut groups = vec![];
        while !ungrouped_groupers.is_empty() {
            let partitioner = ungrouped_groupers
                .get(0)
                .ok_or(SelectError::Unreachable)?
                .1
                .clone();
            let (partition, todo) = ungrouped_groupers
                .into_iter()
                .partition(|(_group_constriant, groupers, _selection)| groupers == &partitioner);
            ungrouped_groupers = todo;
            let partition = partition
                .into_iter()
                .map(|(group_constriant, _, selection)| (group_constriant, selection))
                .collect();
            groups.push(partition);
        }

        groups
            .into_iter()
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
