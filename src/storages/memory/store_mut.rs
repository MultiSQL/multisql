use std::collections::HashMap;

use {
	crate::{MemoryStorage, Result, Row, Schema, StoreMut, Value},
	async_trait::async_trait,
};

#[async_trait(?Send)]
impl StoreMut for MemoryStorage {
	async fn insert_schema(&mut self, schema: &Schema) -> Result<()> {
		let table_name = schema.table_name.clone();
		self.data.insert(table_name.clone(), HashMap::new());
		self.tables.insert(table_name, schema.clone());
		Ok(())
	}

	async fn delete_schema(&mut self, table_name: &str) -> Result<()> {
		self.tables.remove(table_name);
		self.tables.remove(table_name);
		Ok(())
	}

	async fn insert_data(&mut self, table_name: &str, rows: Vec<Row>) -> Result<()> {
		let table_name = table_name.to_string();
		let old_rows = self.data.remove(&table_name).unwrap_or_default();
		let init = old_rows.len();
		let rows = rows
			.into_iter()
			.enumerate()
			.map(|(index, row)| (Value::U64((index + init) as u64), row))
			.chain(old_rows.into_iter())
			.collect();
		self.data.insert(table_name, rows);
		Ok(())
	}

	async fn update_index(
		&mut self,
		table_name: &str,
		index_name: &str,
		keys: Vec<(Value, Value)>,
	) -> Result<()> {
		let (table_name, index_name) = (table_name.to_string(), index_name.to_string());
		let mut indexes = self.indexes.remove(&table_name).unwrap_or_default();
		let mut index = indexes.remove(&index_name).unwrap_or_default();
		index.extend(keys);
		indexes.insert(index_name, index);
		self.indexes.insert(table_name, indexes);
		Ok(())
	}
}
