use {
	crate::{Glue, IndexFilter, Result, Value},
	async_recursion::async_recursion,
};

impl Glue {
	#[async_recursion(?Send)]
	pub async fn get_rows(
		&self,
		table: &str,
		database: &Option<String>,
		index_filter: &Option<IndexFilter>,
	) -> Result<Vec<Vec<Value>>> {
		if let Some((.., context_table_rows)) = self.get_context()?.tables.get(table) {
			Ok(context_table_rows.clone())
		} else {
			let rows = self.get_view_rows(table, database).await?;
			if let Some(rows) = rows {
				Ok(rows)
			} else {
				self.get_table_rows(table, database, index_filter).await
			}
		}
	}
	pub async fn get_view_rows(
		&self,
		view_name: &str,
		database: &Option<String>,
	) -> Result<Option<Vec<Vec<Value>>>> {
		panic!();
		self.get_view_data(view_name, database)
			.await
			.map(|opt| opt.map(|(_, rows)| rows))
	}
	pub async fn get_table_rows(
		&self,
		table: &str,
		database: &Option<String>,
		index_filter: &Option<IndexFilter>,
	) -> Result<Vec<Vec<Value>>> {
		let storage = &**self.get_database(database)?;
		if let Some(index_filter) = index_filter.clone() {
			storage.scan_data_indexed(table, index_filter)
		} else {
			storage.scan_data(table)
		}
		.await
		.map(|plane| {
			plane
				.into_iter()
				.map(|(_, row)| row.0)
				.collect::<Vec<Vec<Value>>>()
		})
	}
}
