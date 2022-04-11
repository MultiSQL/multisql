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
	pub column_defs: Option<HashMap<Option<usize>, Option<Column>>>,
	pub indexes: Option<HashMap<Option<usize>, Option<Index>>>,
}
impl SchemaDiff {
	pub fn new_rename(new_name: String) -> Self {
		Self {
			table_name: Some(new_name),
			column_defs: None,
			indexes: None,
		}
	}
	pub fn new_add_column(new_column: Column) -> Self {
		Self {
			table_name: None,
			column_defs: Some([(None, Some(new_column))].into()),
			indexes: None,
		}
	}
	pub fn new_remove_column(column_index: usize) -> Self {
		Self {
			table_name: None,
			column_defs: Some([(Some(column_index), None)].into()),
			indexes: None,
		}
	}
	pub fn new_rename_column(column_index: usize, mut column: Column, new_column_name: String) -> Self {
		column.name = new_column_name;
		Self {
			table_name: None,
			column_defs: Some([(Some(column_index), Some(column))].into()),
			indexes: None,
		}
	}
	pub fn new_add_index(new_index: Index) -> Self {
		Self {
			table_name: None,
			column_defs: None,
			indexes: Some([(None, Some(new_index))].into()),
		}
	}
}
impl From<Schema> for SchemaDiff {
	fn from(from: Schema) -> Self {
		let column_defs = from.column_defs.into_iter().enumerate().map(|(key, col)| (Some(key), Some(col))).collect::<HashMap<Option<usize>, Option<Column>>>();
		let indexes = from.indexes.into_iter().enumerate().map(|(key, idx)| (Some(key), Some(idx))).collect::<HashMap<Option<usize>, Option<Index>>>();
		Self {
			table_name: Some(from.table_name),
			column_defs: Some(column_defs),
			indexes: Some(indexes),
		}
	}
}