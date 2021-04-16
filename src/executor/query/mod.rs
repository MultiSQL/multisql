mod select;

pub use select::{join::*, ManualError, PlanError, SelectError};

use {
    super::Payload,
    crate::{result::Result, store::Store, Row},
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
}

pub async fn query<'a, Key: 'static + Debug>(
    storage: &'a dyn Store<Key>,
    query: Query,
) -> Result<Payload> {
    match query.body {
        SetExpr::Select(query) => {
            let (labels, rows) = select(storage, *query).await?;

            let rows = rows.into_iter().map(Row).collect(); // I don't like this. TODO

            Ok(Payload::Select { labels, rows })
        }
        _ => Err(QueryError::QueryNotSupported.into()), // TODO: Other queries
    }
}
