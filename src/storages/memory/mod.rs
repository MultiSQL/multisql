mod auto_increment;
mod store;
mod store_mut;

use {
	crate::{store::*, Row, Schema, Value},
	serde::Serialize,
	std::{
		collections::{BTreeMap, HashMap},
		fmt::Debug,
	},
	thiserror::Error,
};

#[derive(Error, Serialize, Debug, PartialEq)]
pub enum MemoryStorageError {
	#[error("table not found")]
	TableNotFound,
}

#[derive(Default, Clone)]
pub struct MemoryStorage {
	tables: HashMap<String, Schema>,
	data: HashMap<String, HashMap<Value, Row>>,
	indexes: HashMap<String, HashMap<String, BTreeMap<Value, Value>>>,
}

impl FullStorage for MemoryStorage {}

impl MemoryStorage {
	pub fn new() -> Self {
		Self::default()
	}
}
