mod store;
mod store_mut;

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
	crate::{data::Row, result::Result, Value},
	serde::{Deserialize, Serialize},
	std::fmt::Debug,
	thiserror::Error,
};

pub use {store::Store, store_mut::StoreMut};

#[derive(Error, Serialize, Debug, PartialEq)]
pub enum StorageError {
	#[error("this storage has not yet implemented this method")]
	Unimplemented,
	#[error("tried to connect to an unknown storage")]
	UnknownConnection,
}

#[derive(Serialize, Deserialize)]
pub enum Connection {
	Unknown,
	#[cfg(feature = "sled-storage")]
	Sled(String),
	#[cfg(feature = "csv-storage")]
	CSV(String, crate::CSVSettings),
	#[cfg(feature = "sheet-storage")]
	Sheet(String),
}
impl Default for Connection {
	fn default() -> Self {
		Connection::Unknown
	}
}
impl TryFrom<Connection> for Storage {
	type Error = crate::Error;
	fn try_from(connection: Connection) -> Result<Storage> {
		use {
			crate::{CSVStorage, SheetStorage, SledStorage},
			Connection::*,
		};
		let storage: Option<Box<dyn FullStorage>> = Some(match &connection {
			#[cfg(feature = "sled-storage")]
			Sled(path) => Box::new(SledStorage::new(path)?),
			#[cfg(feature = "csv-storage")]
			CSV(path, settings) => Box::new(CSVStorage::new_with_settings(path, settings.clone())?),
			#[cfg(feature = "sheet-storage")]
			Sheet(path) => Box::new(SheetStorage::new(path)?),
			Unknown => return Err(StorageError::UnknownConnection.into()),
		});
		Ok(Storage {
			storage,
			source_connection: connection,
		})
	}
}

pub struct Storage {
	source_connection: Connection,
	storage: Option<Box<dyn FullStorage>>,
}
impl Storage {
	pub fn new(storage: Box<dyn FullStorage>) -> Self {
		let storage = Some(storage);
		Self {
			storage,
			source_connection: Connection::default(),
		}
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
	pub fn into_source(self) -> Connection {
		self.source_connection
	}
	pub fn from_source(connection: Connection) -> Result<Self> {
		connection.try_into()
	}
}

pub type StorageInner = dyn FullStorage;

pub trait FullStorage: Store + StoreMut + AlterTable + AutoIncrement {}

pub type RowIter = Box<dyn Iterator<Item = Result<(Value, Row)>>>;
