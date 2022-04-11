use {
	super::{validate, AlterError},
	crate::{data::get_name, Error, Glue, Result},
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

		match operation {
			AlterTableOperation::RenameTable {
				table_name: new_table_name,
			} => {
				let new_table_name = get_name(new_table_name).map_err(Error::from)?;

				database.rename_schema(table_name, new_table_name).await
			}
			AlterTableOperation::RenameColumn {
				old_column_name,
				new_column_name,
			} => {
				database
					.rename_column(table_name, &old_column_name.value, &new_column_name.value)
					.await
			}
			AlterTableOperation::AddColumn { column_def } => {
				validate(column_def).map_err(Error::from)?;

				database.add_column(table_name, &column_def.into()).await
			}
			AlterTableOperation::DropColumn {
				column_name,
				if_exists,
				..
			} => {
				database
					.drop_column(table_name, &column_name.value, *if_exists)
					.await
			}
			_ => Err(AlterError::UnsupportedAlterTableOperation(operation.to_string()).into()),
		}
	}
}
