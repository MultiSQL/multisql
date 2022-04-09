use {
	crate::{IndexFilter, Result, RowIter, Schema, StorageError, MemoryStorageError, Value, MemoryStorage, Store},
	async_trait::async_trait,
};

#[async_trait(?Send)]
impl Store for MemoryStorage {
	async fn fetch_schema(&self, table_name: &str) -> Result<Option<Schema>> {
		Ok(self.tables.get(&table_name.to_string()).cloned())
	}
	async fn scan_schemas(&self) -> Result<Vec<Schema>> {
		Ok(self.tables.values().cloned().collect())
	}

	async fn scan_data(&self, table_name: &str) -> Result<RowIter> {
		let rows = self.data.get(&table_name.to_string()).cloned().ok_or(MemoryStorageError::TableNotFound)?;
		Ok(Box::new(rows.into_iter().map(Ok)))
	}
}
