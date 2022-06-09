mod auto_increment;
mod base;
mod mutable;

use std::sync::{Mutex, MutexGuard};
use {
	crate::Result,
	serde::{Deserialize, Serialize},
	std::fmt::Debug,
	thiserror::Error,
};

pub use {auto_increment::AutoIncrement, base::DBBase, mutable::DBMut};

#[derive(Error, Serialize, Debug, PartialEq)]
pub enum DatabaseError {
	#[error("this database has not yet implemented this method")]
	Unimplemented,
	#[error("tried to connect to an unknown database")]
	UnknownConnection,
	#[error("table not found")]
	TableNotFound,
}

#[derive(Serialize, Deserialize)]
pub enum Connection {
	Unknown,
	#[cfg(feature = "memory-database")]
	Memory,
	#[cfg(feature = "sled-database")]
	Sled(String),
	#[cfg(feature = "csv-database")]
	CSV(String, crate::CSVSettings),
	#[cfg(feature = "sheet-database")]
	Sheet(String),
	#[cfg(feature = "odbc-database")]
	ODBC(String),
}
impl Default for Connection {
	fn default() -> Self {
		Connection::Unknown
	}
}
impl TryFrom<Connection> for Database {
	type Error = crate::Error;
	fn try_from(connection: Connection) -> Result<Database> {
		use Connection::*;
		let database: Mutex<Box<DatabaseInner>> = Mutex::new(match &connection {
			#[cfg(feature = "memory-database")]
			Memory => Box::new(crate::MemoryDatabase::new()),
			#[cfg(feature = "sled-database")]
			Sled(path) => Box::new(crate::SledDatabase::new(path)?),
			#[cfg(feature = "csv-database")]
			CSV(path, settings) => Box::new(crate::CSVDatabase::new_with_settings(
				path,
				settings.clone(),
			)?),
			#[cfg(feature = "sheet-database")]
			Sheet(path) => Box::new(crate::SheetDatabase::new(path)?),
			#[cfg(feature = "odbc-database")]
			ODBC(connection_string) => Box::new(crate::ODBCDatabase::new(connection_string)?),
			Unknown => return Err(DatabaseError::UnknownConnection.into()),
		});
		Ok(Database {
			database,
			source_connection: connection,
		})
	}
}

pub struct Database {
	source_connection: Connection,
	database: Mutex<Box<DatabaseInner>>,
}
impl Database {
	pub fn new(database: Box<DatabaseInner>) -> Self {
		let database = Mutex::new(database);
		Self {
			database,
			source_connection: Connection::default(),
		}
	}
	pub fn get(&self) -> MutexGuard<Box<DatabaseInner>> {
		self.database
			.lock()
			.expect("Unreachable: Database wasn't replaced!")
	}
	pub fn get_mut(&mut self) -> &mut Box<DatabaseInner> {
		self.database
			.get_mut()
			.expect("Unreachable: Database wasn't replaced!")
	}
	pub fn into_source(self) -> Connection {
		self.source_connection
	}
	pub fn from_source(connection: Connection) -> Result<Self> {
		connection.try_into()
	}
}

pub type DatabaseInner = dyn DBFull;

pub trait DBFull: DBBase + DBMut + AutoIncrement {}
