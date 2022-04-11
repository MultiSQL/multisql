use {
	crate::{Column, Index},
	serde::{Deserialize, Serialize},
	std::collections::HashMap,
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
	pub fn new_rename_column(
		column_index: usize,
		mut column: Column,
		new_column_name: String,
	) -> Self {
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

impl SchemaDiff {
	pub fn merge(self, mut schema: Schema) -> Schema {
		if let Some(table_name) = self.table_name {
			schema.table_name = table_name
		}
		if let Some(column_defs) = self.column_defs {
			for (index, column_def) in column_defs.into_iter() {
				match (index, column_def) {
					(None, None) => (),
					(Some(index), None) => {
						schema.column_defs.remove(index);
					} // TODO: WARN: Will be an issue if multiple change
					(Some(index), Some(column_def)) => {
						schema
							.column_defs
							.get_mut(index)
							.map(|old_column_def| *old_column_def = column_def);
					}
					(None, Some(column_def)) => {
						schema.column_defs.push(column_def);
					}
				}
			}
		}
		if let Some(indexes) = self.indexes {
			for (index, index_def) in indexes.into_iter() {
				match (index, index_def) {
					(None, None) => (),
					(Some(index), None) => {
						schema.indexes.remove(index);
					} // TODO: WARN: Will be an issue if multiple change
					(Some(index), Some(index_def)) => {
						schema
							.indexes
							.get_mut(index)
							.map(|old_index_def| *old_index_def = index_def);
					}
					(None, Some(index_def)) => {
						schema.indexes.push(index_def);
					}
				}
			}
		}
		schema
	}
}

impl From<Schema> for SchemaDiff {
	fn from(from: Schema) -> Self {
		let column_defs = from
			.column_defs
			.into_iter()
			.enumerate()
			.map(|(key, col)| (Some(key), Some(col)))
			.collect::<HashMap<Option<usize>, Option<Column>>>();
		let indexes = from
			.indexes
			.into_iter()
			.enumerate()
			.map(|(key, idx)| (Some(key), Some(idx)))
			.collect::<HashMap<Option<usize>, Option<Index>>>();
		Self {
			table_name: Some(from.table_name),
			column_defs: Some(column_defs),
			indexes: Some(indexes),
		}
	}
}
