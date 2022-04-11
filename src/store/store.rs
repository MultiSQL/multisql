use {
	crate::{IndexFilter, Result, RowIter, Schema, StorageError, Value},
	async_trait::async_trait,
};

/// `Store` -> `SELECT`
#[async_trait(?Send)]
pub trait Store {
	async fn fetch_schema(&self, _table_name: &str) -> Result<Option<Schema>> {
		Err(StorageError::Unimplemented.into())
	}
	async fn scan_schemas(&self) -> Result<Vec<Schema>> {
		Err(StorageError::Unimplemented.into())
	}

	async fn scan_data(&self, _table_name: &str) -> Result<RowIter> {
		Err(StorageError::Unimplemented.into())
	}

	// TODO: Move to own trait
	async fn scan_data_indexed(
		&self,
		_table_name: &str,
		_index_filters: IndexFilter,
	) -> Result<RowIter> {
		Err(StorageError::Unimplemented.into())
	}
	async fn scan_index(
		&self,
		_table_name: &str,
		_index_filter: IndexFilter,
	) -> Result<Vec<Value>> {
		Err(StorageError::Unimplemented.into())
	}
}
