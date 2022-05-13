use crate::{
	executor::{
		fetch::fetch_columns,
		types::{ColumnInfo, ComplexTableName},
	},
	Glue, Result,
};

impl Glue {
	pub async fn get_columns(&self, table: ComplexTableName) -> Result<Vec<ColumnInfo>> {
		if let Some((context_table_labels, ..)) = self.get_context()?.tables.get(&table.name) {
			Ok(context_table_labels
				.iter()
				.map(|name| ColumnInfo {
					table: table.clone(),
					name: name.clone(),
					index: None,
				})
				.collect::<Vec<ColumnInfo>>())
		} else {
			fetch_columns(&**self.get_database(&table.database)?, table).await
		}
	}
}
