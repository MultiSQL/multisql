use {
	super::AlterError,
	crate::{
		data::{get_name, schema::ColumnDefExt},
		Result, StorageInner,
	},
	futures::stream::{self, TryStreamExt},
	sqlparser::ast::ObjectName,
};

pub async fn truncate(storage: &mut StorageInner, table_name: &ObjectName) -> Result<()> {
	let table_name = get_name(table_name)?;
	let schema = storage.fetch_schema(table_name).await?;

	if let Some(schema) = schema {
		// TODO: We should be deleting the entry
		#[cfg(feature = "auto-increment")]
		let result: Result<&mut StorageInner> = stream::iter(schema.column_defs.iter().map(Ok))
			.try_fold(storage, |storage, column| async move {
				if column.is_auto_incremented() {
					storage
						.set_increment_value(table_name, column.name.value.as_str(), 1_i64)
						.await?;
				}
				Ok(storage)
			})
			.await;

		#[cfg(feature = "auto-increment")]
		let storage = result?;

		// TODO: Maybe individual "truncate" operation
		storage.delete_schema(table_name).await?; // TODO: !!! This will delete INDEXes which it shouldn't!
		storage.insert_schema(&schema).await?;
		Ok(())
	} else {
		Err(AlterError::TableNotFound(table_name.to_owned()).into())
	}
}
