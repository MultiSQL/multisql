use {
    crate::{result::Result, StorageInner},
    serde::Serialize,
    sqlparser::ast::{ColumnDef, Ident},
    thiserror::Error as ThisError,
};

#[derive(ThisError, Serialize, Debug, PartialEq)]
pub enum FetchError {
    #[error("table not found: {0}")]
    TableNotFound(String),
}

pub async fn fetch_columns(storage: &StorageInner, table_name: &str) -> Result<Vec<Ident>> {
    Ok(storage
        .fetch_schema(table_name)
        .await?
        .ok_or_else(|| FetchError::TableNotFound(table_name.to_string()))?
        .column_defs
        .into_iter()
        .map(|ColumnDef { name, .. }| name)
        .collect::<Vec<Ident>>())
}
