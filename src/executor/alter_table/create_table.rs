use {
	super::{validate, AlterError},
	crate::{
		data::{get_name, Schema},
		Result, StorageInner,
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
		column_defs: column_defs.to_vec(),
	};

	for column_def in &schema.column_defs {
		validate(column_def)?;
	}

	if let Some(_) = storage.fetch_schema(&schema.table_name).await? {
		if !if_not_exists {
			Err(AlterError::TableAlreadyExists(schema.table_name.to_owned()).into())
		} else {
			Ok(())
		}
	} else {
		storage.insert_schema(&schema).await
	}
}
