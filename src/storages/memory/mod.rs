mod alter_table;
mod auto_increment;
mod store;
mod store_mut;

use {
	crate::{store::*, Value, Schema, Row},
	serde::Serialize,
	std::{fmt::Debug, collections::HashMap},
	thiserror::Error,
};

#[derive(Error, Serialize, Debug, PartialEq)]
pub enum MemoryStorageError {
	#[error("table not found")]
	TableNotFound
}

#[derive(Default, Clone)]
pub struct MemoryStorage {
	tables: HashMap<String, Schema>,
	data: HashMap<String, HashMap<Value, Row>>,
}

impl FullStorage for MemoryStorage {}

impl MemoryStorage {
	pub fn new() -> Self {
		Self::default()
	}
}
