use {
	crate::{Column, Result, Schema, MemoryStorage, StorageError, AlterTable},
	async_trait::async_trait,
};

#[async_trait(?Send)]
impl AlterTable for MemoryStorage {
	async fn rename_schema(&mut self, _table_name: &str, _new_table_name: &str) -> Result<()> {
		Err(StorageError::Unimplemented.into())
	}

	async fn rename_column(
		&mut self,
		_table_name: &str,
		_old_column_name: &str,
		_new_column_name: &str,
	) -> Result<()> {
		Err(StorageError::Unimplemented.into())
	}
	async fn add_column(&mut self, _table_name: &str, _column: &Column) -> Result<()> {
		Err(StorageError::Unimplemented.into())
	}
	async fn drop_column(
		&mut self,
		_table_name: &str,
		_column_name: &str,
		_if_exists: bool,
	) -> Result<()> {
		Err(StorageError::Unimplemented.into())
	}
	async fn replace_schema(&mut self, _table_name: &str, _schema: Schema) -> Result<()> {
		Err(StorageError::Unimplemented.into())
	}
}
