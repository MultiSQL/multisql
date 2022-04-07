use {
	crate::{Result, Row, Schema, StorageError, Value},
	async_trait::async_trait,
};

/// `StoreMut` -> `INSERT`, `CREATE`, `DELETE`, `DROP`, `UPDATE`
#[async_trait(?Send)]
pub trait StoreMut {
	async fn insert_schema(&mut self, _schema: &Schema) -> Result<()> {
		Err(StorageError::Unimplemented.into())
	}

	async fn delete_schema(&mut self, _table_name: &str) -> Result<()> {
		Err(StorageError::Unimplemented.into())
	} // Shouldn't this be AlterTable?

	async fn insert_data(&mut self, _table_name: &str, _rows: Vec<Row>) -> Result<()> {
		Err(StorageError::Unimplemented.into())
	}

	async fn update_data(&mut self, _table_name: &str, _rows: Vec<(Value, Row)>) -> Result<()> {
		Err(StorageError::Unimplemented.into())
	}

	async fn delete_data(&mut self, _table_name: &str, _keys: Vec<Value>) -> Result<()> {
		Err(StorageError::Unimplemented.into())
	}

	async fn update_index(
		&mut self,
		_index_name: &str,
		_table_name: &str,
		_keys: Vec<(Value, Value)>,
	) -> Result<()> {
		Err(StorageError::Unimplemented.into())
	}
}
