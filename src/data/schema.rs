use {
	crate::{Column, Index},
	serde::{Deserialize, Serialize},
	std::collections::HashMap
};

#[derive(Clone, Serialize, Deserialize)]
pub struct Schema {
	pub table_name: String,
	pub column_defs: Vec<Column>,
	pub indexes: Vec<Index>,
}

#[derive(Clone, Default)]
pub struct SchemaDiff {
	pub table_name: Option<String>,
	pub column_defs: Option<HashMap<usize, Option<Column>>>,
	pub indexes: Option<HashMap<usize, Option<Index>>>,
}

impl From<Schema> for SchemaDiff {
	fn from(from: Schema) -> Self {
		let column_defs = from.column_defs.into_iter().map(Some).enumerate().collect::<HashMap<usize, Option<Column>>>();
		let indexes = from.indexes.into_iter().map(Some).enumerate().collect::<HashMap<usize, Option<Index>>>();
		Self {
			table_name: Some(from.table_name),
			column_defs: Some(column_defs),
			indexes: Some(indexes),
		}
	}
}