use crate::ExecuteError;

use {
	super::{validate, AlterError},
	crate::{
		Index,
		data::get_name,
		Result, StorageInner,
	},
	sqlparser::ast::{OrderByExpr, ObjectName},
};

pub async fn create_index(
	storage: &mut StorageInner,
	table: &ObjectName,
	name: &ObjectName,
	columns: &[OrderByExpr],
	unique: bool,
	if_not_exists: bool,
) -> Result<()> {
	let name = name.0.last().ok_or(Err(ExecuteError::QueryNotSupported))?.value.clone();

	let table_name = get_name(table)?;

	let schema = storage.fetch_schema(&table_name).await?.ok_or(Err(ExecuteError::TableNotExists))?;

	if let Some(_) = schema.indexes.iter().find(|index| index.name == name) {
		if !if_not_exists {
			Err(AlterError::AlreadyExists(name).into())
		} else {
			Ok(())
		}
	} else {
		let schema = schema.clone();
		let index = Index::new(name, columns, unique)?;
		index.reset(storage, &table_name, &schema.column_defs).await;
		schema.indexes.push(index);
		storage.replace_schema(&table_name, schema).await
	}
}
