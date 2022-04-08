use {
	super::AlterError,
	crate::{data::get_name, Glue, Result, StorageInner, ValueDefault},
	futures::stream::{self, TryStreamExt},
	sqlparser::ast::{ObjectName, ObjectType},
};

impl Glue {
	pub async fn drop(
		&mut self,
		object_type: &ObjectType,
		names: &[ObjectName],
		if_exists: bool,
	) -> Result<()> {
		let database = &mut **self.get_mut_database(&None)?;
		if object_type != &ObjectType::Table {
			return Err(AlterError::DropTypeNotSupported(object_type.to_string()).into());
		}

		stream::iter(names.iter().map(Ok))
			.try_fold(database, |database, table_name| async move {
				let table_name = get_name(table_name)?;
				let schema = database.fetch_schema(table_name).await?;

				if schema.is_none() {
					if !if_exists {
						return Err(AlterError::TableNotFound(table_name.to_owned()).into());
					} else {
						return Ok(database);
					}
				}
				#[cfg(feature = "auto-increment")]
				let result: Result<&mut StorageInner> =
					stream::iter(schema.unwrap().column_defs.into_iter().map(Ok))
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

				database.delete_schema(table_name).await?;
				Ok(database)
			})
			.await
			.map(|_| ())
	}
}
