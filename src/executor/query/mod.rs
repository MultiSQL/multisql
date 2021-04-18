mod select;

pub use select::{join::*, ManualError, PlanError, SelectError};

use {
    crate::{
        executor::types::LabelsAndRows, macros::warning, result::Result, store::Store, Cast,
        Recipe, RecipeUtilities, Resolve, SimplifyBy, Value,
    },
    select::select,
    serde::Serialize,
    sqlparser::ast::{Query, SetExpr},
    std::fmt::Debug,
    thiserror::Error as ThisError,
};

const ENSURE_SIZE: bool = true;

#[derive(ThisError, Serialize, Debug, PartialEq)]
pub enum QueryError {
    #[error("query not supported")]
    QueryNotSupported,
    #[error("values does not support columns, aggregates or subqueries")]
    MissingComponentsForValues,
    #[error("literal does not support columns, aggregates or subqueries")]
    MissingComponentsForLimit,
    #[error("expected values but found none")]
    NoValues,
}

pub async fn query<'a, Key: 'static + Debug>(
    storage: &'a dyn Store<Key>,
    query: Query,
) -> Result<LabelsAndRows> {
    let Query {
        body,
        order_by,
        limit,
        // TODO (below)
        offset: _,
        fetch: _,
        with: _,
    } = query;
    let limit: Option<usize> = limit
        .map(|expression| {
            Recipe::new_without_meta(expression)?
                .simplify_by_basic()?
                .confirm_or_err(QueryError::MissingComponentsForLimit.into())?
                .cast()
        })
        .transpose()?;
    let (mut labels, mut rows) = match body {
        SetExpr::Select(query) => {
            let (labels, rows) = select(storage, *query, order_by).await?;

            Ok((labels, rows))
        }
        SetExpr::Values(values) => {
            if !order_by.is_empty() {
                warning!("VALUES does not currently support ordering");
            }
            let values = values.0;
            values
                .into_iter()
                .map(|values_row| {
                    values_row
                        .into_iter()
                        .map(|cell| {
                            Recipe::new_without_meta(cell)?
                                .simplify_by_basic()?
                                .confirm_or_err(QueryError::MissingComponentsForValues.into())
                        })
                        .collect::<Result<Vec<Value>>>()
                })
                .collect::<Result<Vec<Vec<Value>>>>()
                .map(|values| {
                    (
                        vec![
                            String::new();
                            values.get(0).map(|first_row| first_row.len()).unwrap_or(0)
                        ],
                        values,
                    )
                })
        }
        _ => Err(QueryError::QueryNotSupported.into()), // TODO: Other queries
    }?;

    limit.map(|limit| rows.truncate(limit));
    if ENSURE_SIZE {
        let row_width = rows
            .iter()
            .map(|values_row| values_row.len())
            .max()
            .unwrap_or(0);
        if row_width > 0 {
            rows = rows
                .into_iter()
                .map(|mut row| {
                    row.resize(row_width, Value::Null);
                    row
                })
                .collect();
            labels.resize(row_width, String::new())
        };
    }
    Ok((labels, rows))
}
