mod select;

pub use select::{join::*, ManualError, PlanError, SelectError};

use {
    crate::{
        executor::types::LabelsAndRows, result::Result, store::Store, Recipe, RecipeUtilities,
        Resolve, SimplifyBy, Value,
    },
    select::select,
    serde::Serialize,
    sqlparser::ast::{Query, SetExpr},
    std::fmt::Debug,
    thiserror::Error as ThisError,
};

#[derive(ThisError, Serialize, Debug, PartialEq)]
pub enum QueryError {
    #[error("query not supported")]
    QueryNotSupported,
    #[error("values does not support columns, aggregates or subqueries")]
    MissingComponentsForValues,
}

pub async fn query<'a, Key: 'static + Debug>(
    storage: &'a dyn Store<Key>,
    query: Query,
) -> Result<LabelsAndRows> {
    match query.body {
        SetExpr::Select(query) => {
            let (labels, rows) = select(storage, *query).await?;

            Ok((labels, rows))
        }
        SetExpr::Values(values) => values
            .0
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|cell| {
                        Recipe::new_without_meta(cell)?
                            .simplify(SimplifyBy::Basic)?
                            .as_solution()
                            .ok_or(QueryError::MissingComponentsForValues.into())
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
            }),
        _ => Err(QueryError::QueryNotSupported.into()), // TODO: Other queries
    }
}
