use super::types::{ColumnInfo, ComplexTableName};

use {
	crate::{result::Result, StorageInner},
	serde::Serialize,
	sqlparser::ast::ColumnDef,
	thiserror::Error as ThisError,
};

#[derive(ThisError, Serialize, Debug, PartialEq)]
pub enum FetchError {
	#[error("table not found: {0}")]
	TableNotFound(String),
}

pub async fn fetch_columns(
	storage: &StorageInner,
	table: ComplexTableName,
) -> Result<Vec<ColumnInfo>> {
	let schema = storage
		.fetch_schema(&table.name)
		.await?
		.ok_or_else(|| FetchError::TableNotFound(table.name.clone()))?;
	let columns = schema
		.column_defs
		.iter()
		.map(|ColumnDef { name, .. }| {
			let name = name.value.clone();
			let index = schema
				.indexes
				.iter()
				.find_map(|index| (index.column == name).then(|| index.name.clone()));
			ColumnInfo {
				table: table.clone(),
				name,
				index,
			}
		})
		.collect();
	Ok(columns)
}
