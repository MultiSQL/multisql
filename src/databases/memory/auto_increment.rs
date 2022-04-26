use {
	crate::{AutoIncrement, MemoryDatabase, Result},
	async_trait::async_trait,
};

#[async_trait(?Send)]
impl AutoIncrement for MemoryDatabase {
	async fn generate_increment_values(
		&mut self,
		table_name: String,
		columns: Vec<(usize, String, i64)>,
	) -> Result<Vec<((usize, String), i64)>> {
		let row_init = self
			.data
			.get(&table_name)
			.map(|rows| rows.len() + 1)
			.unwrap_or(1);
		Ok(columns
			.into_iter()
			.map(|(index, name, _)| ((index, name), row_init as i64))
			.collect())
	}

	async fn set_increment_value(
		&mut self,
		_table_name: &str,
		_column_name: &str,
		_end: i64,
	) -> Result<()> {
		Ok(())
	}
}
