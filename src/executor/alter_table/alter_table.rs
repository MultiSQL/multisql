use {
	super::{validate, AlterError},
	crate::{data::get_name, Error, Glue, Result, SchemaDiff},
	sqlparser::ast::{AlterTableOperation, ObjectName},
};

impl Glue {
	pub async fn alter_table(
		&mut self,
		name: &ObjectName,
		operation: &AlterTableOperation,
	) -> Result<()> {
		let table_name = get_name(name).map_err(Error::from)?;
		let database = &mut **self.get_mut_database(&None)?;

		let diff = match operation {
			AlterTableOperation::RenameTable {
				table_name: new_table_name,
			} => {
				let new_table_name = get_name(new_table_name).map_err(Error::from)?;

				SchemaDiff::new_rename(new_table_name.clone())
			}
			AlterTableOperation::RenameColumn {
				old_column_name,
				new_column_name,
			} => {
				let schema = database
					.fetch_schema(table_name)
					.await?
					.ok_or(AlterError::TableNotFound(table_name.clone()))?;
				let (column_index, column) = schema
					.column_defs
					.into_iter()
					.enumerate()
					.find(|(_, column)| column.name == old_column_name.value)
					.ok_or(AlterError::ColumnNotFound(
						table_name.clone(),
						old_column_name.value.clone(),
					))?;
				SchemaDiff::new_rename_column(column_index, column, new_column_name.value.clone())
			}
			AlterTableOperation::AddColumn { column_def } => {
				validate(column_def).map_err(Error::from)?;

				SchemaDiff::new_add_column(column_def.into())
			}
			AlterTableOperation::DropColumn {
				column_name,
				if_exists,
				..
			} => {
				let schema = database
					.fetch_schema(table_name)
					.await?
					.ok_or(AlterError::TableNotFound(table_name.clone()))?;
				let (column_index, _) = schema
					.column_defs
					.into_iter()
					.enumerate()
					.find(|(_, column)| column.name == column_name.value)
					.ok_or(AlterError::ColumnNotFound(
						table_name.clone(),
						column_name.value.clone(),
					))?;

				SchemaDiff::new_remove_column(column_index)
			}
			_ => {
				return Err(
					AlterError::UnsupportedAlterTableOperation(operation.to_string()).into(),
				)
			}
		};
		database.alter_table(table_name, diff).await
	}
}
