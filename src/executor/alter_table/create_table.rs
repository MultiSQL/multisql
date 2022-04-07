use {
	super::{validate, AlterError},
	crate::{
		data::{get_name, Schema},
		Column, Result, StorageInner,
	},
	sqlparser::ast::{ColumnDef, ObjectName},
};

pub async fn create_table(
	storage: &mut StorageInner,
	name: &ObjectName,
	column_defs: &[ColumnDef],
	if_not_exists: bool,
) -> Result<()> {
	let schema = Schema {
		table_name: get_name(name)?.to_string(),
		column_defs: column_defs.iter().cloned().map(Column::from).collect(),
		indexes: vec![],
	};

	if storage.fetch_schema(&schema.table_name).await?.is_some() {
		if !if_not_exists {
			Err(AlterError::TableAlreadyExists(schema.table_name.to_owned()).into())
		} else {
			Ok(())
		}
	} else {
		storage.insert_schema(&schema).await
	}
}
