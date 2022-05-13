use crate::{Glue, IndexFilter, Result, Value};

impl Glue {
	pub async fn get_rows(
		&self,
		table: &str,
		database: &Option<String>,
		index_filter: &Option<IndexFilter>,
	) -> Result<Vec<Vec<Value>>> {
		if let Some((.., context_table_rows)) = self.get_context()?.tables.get(table) {
			Ok(context_table_rows.clone())
		} else {
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
}
