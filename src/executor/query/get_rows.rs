use {crate::{Cast, Glue, IndexFilter, Result, Value, VIEW_TABLE_NAME},
async_recursion::async_recursion};

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
		let views = self
			.get_table_rows(VIEW_TABLE_NAME, database, &None)
			.await?;
		let query = views.into_iter().find_map(|row| {
			let name = row.get(0);
			if name == Some(&Value::Str(view_name.to_string())) {
				Some(row[1].clone())
			} else {
				None
			}
		});
		if let Some(query) = query {
			let query: String = query.clone().cast()?;
			let query = serde_yaml::from_str(&query).unwrap(); // TODO: Handle
			self.no_cte_query(query)
				.await
				.map(|(labels, rows)| rows)
				.map(Some)
		} else {
			Ok(None)
		}
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
