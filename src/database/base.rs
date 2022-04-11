use {
	crate::{IndexFilter, Plane, Result, Schema, DatabaseError, Value},
	async_trait::async_trait,
};

/// `Store` -> `SELECT`
#[async_trait(?Send)]
pub trait DBBase {
	async fn fetch_schema(&self, _table_name: &str) -> Result<Option<Schema>> {
		Err(DatabaseError::Unimplemented.into())
	}
	async fn scan_schemas(&self) -> Result<Vec<Schema>> {
		Err(DatabaseError::Unimplemented.into())
	}

	async fn scan_data(&self, _table_name: &str) -> Result<Plane> {
		Err(DatabaseError::Unimplemented.into())
	}

	async fn scan_data_indexed(
		&self,
		_table_name: &str,
		_index_filters: IndexFilter,
	) -> Result<Plane> {
		Err(DatabaseError::Unimplemented.into())
	}
	async fn scan_index(
		&self,
		_table_name: &str,
		_index_filter: IndexFilter,
	) -> Result<Vec<Value>> {
		Err(DatabaseError::Unimplemented.into())
	}
}
