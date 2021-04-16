use boolinator::Boolinator;
use futures::stream::{self, TryStream, TryStreamExt};
use serde::Serialize;
use std::fmt::Debug;
use std::rc::Rc;
use thiserror::Error as ThisError;

use sqlparser::ast::{ColumnDef, Ident};

use crate::data::Row;
use crate::result::{Error, Result};
use crate::store::Store;

#[derive(ThisError, Serialize, Debug, PartialEq)]
pub enum FetchError {
    #[error("table not found: {0}")]
    TableNotFound(String),
}

pub async fn fetch_columns<T: 'static + Debug>(
    storage: &dyn Store<T>,
    table_name: &str,
) -> Result<Vec<Ident>> {
    Ok(storage
        .fetch_schema(table_name)
        .await?
        .ok_or_else(|| FetchError::TableNotFound(table_name.to_string()))?
        .column_defs
        .into_iter()
        .map(|ColumnDef { name, .. }| name)
        .collect::<Vec<Ident>>())
}
