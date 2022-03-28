use std::cmp::Ordering;

use crate::{result::Result, Row, StorageInner, Value};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use sqlparser::ast::ColumnDef;

#[derive(Clone, Serialize, Deserialize)]
pub struct Index {
	pub name: String,
	pub column: String,
	pub is_unique: bool,
}

#[derive(Clone)]
pub enum IndexFilter {
	Between(String, Value, Value), // Index, Min, Max
	Inner(Box<IndexFilter>, Box<IndexFilter>),
	Outer(Box<IndexFilter>, Box<IndexFilter>),
}

impl Index {
	pub fn new(name: String, column: String, is_unique: bool) -> Self {
		Self {
			name,
			column,
			is_unique,
		}
	}
	pub async fn reset(
		&self,
		storage: &mut StorageInner,
		table: &str,
		column_defs: &[ColumnDef],
	) -> Result<()> {
		let rows = storage
			.scan_data(table)
			.await?
			.collect::<Result<Vec<(Value, Row)>>>()?;
		let column_index: usize = column_defs
			.iter()
			.enumerate()
			.find_map(|(index, def)| (def.name.value == self.column).then(|| index))
			.unwrap(); // TODO: Handle

		let mut rows: Vec<(Value, Vec<Value>)> =
			rows.into_iter().map(|(key, row)| (key, row.0)).collect();
		rows.par_sort_unstable_by(|(_, a_values), (_, b_values)| {
			a_values[column_index]
				.partial_cmp(&b_values[column_index])
				.unwrap_or(Ordering::Equal)
		});
		let keys = rows
			.into_iter()
			.map(|(key, mut values)| (values.swap_remove(column_index), key))
			.collect();

		storage.update_index(&table, &self.name, keys).await
	}
}
