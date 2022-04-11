use {
	crate::{AlterTable, Column, MemoryStorage, Result, Schema, StorageError},
	async_trait::async_trait,
};

#[async_trait(?Send)]
impl AlterTable for MemoryStorage {
	async fn rename_schema(&mut self, old_table_name: &str, new_table_name: &str) -> Result<()> {
		let old_table_name = old_table_name.to_string();
		let new_table_name = new_table_name.to_string();
		let mut schema = self.tables.remove(&old_table_name).unwrap(); // TODO: Handle
		let data = self.data.remove(&old_table_name).unwrap_or_default();

		schema.table_name = new_table_name.clone();

		self.data.insert(new_table_name.clone(), data);
		self.tables.insert(new_table_name, schema);
		
		Ok(())
	}
	async fn replace_schema(&mut self, table_name: &str, schema: Schema) -> Result<()> {
		self.tables.remove(&table_name.to_string());
		let data = self.data.remove(&table_name.to_string()).unwrap_or_default();

		let table_name = schema.table_name.clone();
		self.data.insert(table_name.clone(), data);
		self.tables.insert(table_name, schema);

		Ok(())
	}	
}
