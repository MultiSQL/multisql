use {
	crate::{
		result::Result,
		types::{ColumnInfo, ComplexTableName},
		Column, DatabaseInner,
	},
	serde::Serialize,
	thiserror::Error as ThisError,
};

#[derive(ThisError, Serialize, Debug, PartialEq)]
pub enum FetchError {
	#[error("table not found: {0}")]
	TableNotFound(String),
}

pub async fn fetch_columns(
	storage: &DatabaseInner,
	table: ComplexTableName,
) -> Result<Vec<ColumnInfo>> {
	let schema = storage
		.fetch_schema(&table.name)
		.await?
		.ok_or_else(|| FetchError::TableNotFound(table.name.clone()))?;
	let columns = schema
		.column_defs
		.iter()
		.map(|Column { name, .. }| {
			let index = schema
				.indexes
				.iter()
				.find_map(|index| (&index.column == name).then(|| index.name.clone()));
			ColumnInfo {
				table: table.clone(),
				name: name.clone(),
				index,
			}
		})
		.collect();
	Ok(columns)
}
