use {
	super::AlterError,
	crate::{data::get_name, Result, StorageInner, ValueDefault},
	futures::stream::{self, TryStreamExt},
	sqlparser::ast::{ObjectName, ObjectType},
};

pub async fn drop(
	storage: &mut StorageInner,
	object_type: &ObjectType,
	names: &[ObjectName],
	if_exists: bool,
) -> Result<()> {
	if object_type != &ObjectType::Table {
		return Err(AlterError::DropTypeNotSupported(object_type.to_string()).into());
	}

	stream::iter(names.iter().map(Ok))
		.try_fold(storage, |storage, table_name| async move {
			let table_name = get_name(table_name)?;
			let schema = storage.fetch_schema(table_name).await?;

			if schema.is_none() {
				if !if_exists {
					return Err(AlterError::TableNotFound(table_name.to_owned()).into());
				} else {
					return Ok(storage);
				}
			}
			#[cfg(feature = "auto-increment")]
			let result: Result<&mut StorageInner> =
				stream::iter(schema.unwrap().column_defs.into_iter().map(Ok))
					.try_fold(storage, |storage, column| async move {
						if matches!(column.default, Some(ValueDefault::AutoIncrement(_))) {
							storage
								.set_increment_value(table_name, &column.name, 1_i64)
								.await?;
						}
						Ok(storage)
					})
					.await;

			#[cfg(feature = "auto-increment")]
			let storage = result?;

			storage.delete_schema(table_name).await?;
			Ok(storage)
		})
		.await
		.map(|_| ())
}
