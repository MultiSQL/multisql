use {
	crate::{data::get_name, AlterError, DatabaseInner, Glue, Result, ValueDefault},
	futures::stream::{self, TryStreamExt},
	sqlparser::ast::ObjectName,
};

impl Glue {
	pub async fn ast_truncate(&mut self, table_name: &ObjectName) -> Result<()> {
		let database = &mut **self.get_mut_database(&None)?;
		let table_name = get_name(table_name)?;
		let schema = database.fetch_schema(table_name).await?;

		if let Some(schema) = schema {
			// TODO: We should be deleting the entry
			#[cfg(feature = "auto-increment")]
			let result: Result<&mut DatabaseInner> = stream::iter(schema.column_defs.iter().map(Ok))
				.try_fold(database, |database, column| async move {
					if matches!(column.default, Some(ValueDefault::AutoIncrement(_))) {
						database
							.set_increment_value(table_name, &column.name, 1_i64)
							.await?;
					}
					Ok(database)
				})
				.await;

			#[cfg(feature = "auto-increment")]
			let database = result?;

			// TODO: Maybe individual "truncate" operation
			database.delete_schema(table_name).await?; // TODO: !!! This will delete INDEXes which it shouldn't!
			database.insert_schema(&schema).await?;
			Ok(())
		} else {
			Err(AlterError::TableNotFound(table_name.to_owned()).into())
		}
	}
}
