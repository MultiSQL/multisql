use {
	super::AlterError,
	crate::{ComplexTableName, Glue, Result, ValueDefault},
	sqlparser::ast::{ObjectName, ObjectType},
};

impl Glue {
	pub async fn drop(
		&mut self,
		object_type: &ObjectType,
		names: &[ObjectName],
		if_exists: bool,
	) -> Result<()> {
		if object_type != &ObjectType::Table {
			return Err(AlterError::DropTypeNotSupported(object_type.to_string()).into());
		}

		for name in names.iter() {
			let ComplexTableName {
				name: table_name,
				database,
				..
			} = name.try_into()?;

			let database = &mut **self.get_mut_database(&database)?;
			let schema = database.fetch_schema(&table_name).await?;

			if let Some(schema) = schema {
				for column in schema.column_defs {
					if matches!(column.default, Some(ValueDefault::AutoIncrement(_))) {
						database
							.set_increment_value(&table_name, &column.name, 1_i64)
							.await?;
					}
				}

				database.delete_schema(&table_name).await?;
			} else if !if_exists {
				return Err(AlterError::TableNotFound(table_name.to_owned()).into());
			}
		}
		Ok(())
	}
}
