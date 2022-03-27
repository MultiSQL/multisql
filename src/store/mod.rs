#[cfg(feature = "alter-table")]
mod alter_table;
#[cfg(feature = "alter-table")]
pub use alter_table::*;
#[cfg(not(feature = "alter-table"))]
pub trait AlterTable {}

#[cfg(feature = "auto-increment")]
mod auto_increment;
#[cfg(feature = "auto-increment")]
pub use auto_increment::AutoIncrement;
#[cfg(not(feature = "auto-increment"))]
pub trait AutoIncrement {}

use {
	crate::{
		data::{Row, Schema},
		result::Result,
		Value,
	},
	async_trait::async_trait,
	serde::Serialize,
	std::fmt::Debug,
	thiserror::Error,
};

#[derive(Error, Serialize, Debug, PartialEq)]
pub enum StorageError {
	#[error("this storage has not yet implemented this method")]
	Unimplemented,
}

pub struct Storage {
	storage: Option<Box<dyn FullStorage>>,
}
impl Storage {
	pub fn new(storage: Box<dyn FullStorage>) -> Self {
		let storage = Some(storage);
		Self { storage }
	}
	pub fn replace(&mut self, storage: Box<dyn FullStorage>) {
		self.storage.replace(storage);
	}
	pub fn take(&mut self) -> Box<dyn FullStorage> {
		self.storage
			.take()
			.expect("Unreachable: Storage wasn't replaced!")
	}
	pub fn take_readable(&mut self) -> &StorageInner {
		/*let storage = self.take();
		let readable = &*storage;
		self.replace(storage);
		readable*/
		unimplemented!()
	}
}

pub type StorageInner = dyn FullStorage;

pub trait FullStorage: Store + StoreMut + AlterTable + AutoIncrement {}

pub type RowIter = Box<dyn Iterator<Item = Result<(Value, Row)>>>;

/// By implementing `Store` trait, you can run `SELECT` queries.
#[async_trait(?Send)]
pub trait Store {
	async fn fetch_schema(&self, _table_name: &str) -> Result<Option<Schema>> {
		Err(StorageError::Unimplemented.into())
	}

	async fn scan_data(&self, _table_name: &str) -> Result<RowIter> {
		Err(StorageError::Unimplemented.into())
	}
}

/// `StoreMut` takes role of mutation, related to `INSERT`, `CREATE`, `DELETE`, `DROP` and
/// `UPDATE`.
#[async_trait(?Send)]
pub trait StoreMut {
	async fn insert_schema(&mut self, _schema: &Schema) -> Result<()> {
		Err(StorageError::Unimplemented.into())
	}

	async fn delete_schema(&mut self, _table_name: &str) -> Result<()> {
		Err(StorageError::Unimplemented.into())
	}

	async fn insert_data(&mut self, _table_name: &str, _rows: Vec<Row>) -> Result<()> {
		Err(StorageError::Unimplemented.into())
	}

	async fn update_data(&mut self, _rows: Vec<(Value, Row)>) -> Result<()> {
		Err(StorageError::Unimplemented.into())
	}

	async fn delete_data(&mut self, _keys: Vec<Value>) -> Result<()> {
		Err(StorageError::Unimplemented.into())
	}

	async fn update_index(&mut self, _index_name: &str, _table_name: &str, _keys: Vec<Value>) -> Result<()> {
		Err(StorageError::Unimplemented.into())
	}
}
