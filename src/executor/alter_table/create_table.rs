use {
	super::AlterError,
	crate::{
		data::{get_name, Schema},
		Column, Glue, Result,
	},
	sqlparser::ast::{ColumnDef, ObjectName},
};

impl Glue {
	pub async fn create_table(
		&mut self,
		name: &ObjectName,
		column_defs: &[ColumnDef],
		if_not_exists: bool,
	) -> Result<()> {
		let schema = Schema {
			table_name: get_name(name)?.to_string(),
			column_defs: column_defs.iter().cloned().map(Column::from).collect(),
			indexes: vec![],
		};

		let database = self.get_mut_database(&None)?;
		if database.fetch_schema(&schema.table_name).await?.is_some() {
			if !if_not_exists {
				Err(AlterError::TableAlreadyExists(schema.table_name.to_owned()).into())
			} else {
				Ok(())
			}
		} else {
			database.insert_schema(&schema).await
		}
	}
}
