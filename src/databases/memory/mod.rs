mod auto_increment;
mod base;
mod mutable;

use {
	crate::{database::*, Row, Schema, Value},
	serde::Serialize,
	std::{
		collections::{BTreeMap, HashMap},
		fmt::Debug,
	},
	thiserror::Error,
};

#[derive(Error, Serialize, Debug, PartialEq)]
pub enum MemoryDatabaseError {
	#[error("table not found")]
	TableNotFound,
}

#[derive(Default, Clone)]
pub struct MemoryDatabase {
	tables: HashMap<String, Schema>,
	data: HashMap<String, HashMap<Value, Row>>,
	indexes: HashMap<String, HashMap<String, BTreeMap<Value, Value>>>,
}

impl DBFull for MemoryDatabase {}

impl MemoryDatabase {
	pub fn new() -> Self {
		Self::default()
	}
}
