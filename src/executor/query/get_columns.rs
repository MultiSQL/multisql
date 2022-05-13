use {
	super::Plan,
	crate::{
		executor::{
			fetch::fetch_columns,
			types::{ColumnInfo, ComplexTableName},
		},
		Glue, Result,
	},
	async_recursion::async_recursion,
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
			let labels = self.get_view_columns(&table.name, &table.database).await?;
			if let Some(labels) = labels {
				let labels = labels
					.into_iter()
					.map(|name| ColumnInfo {
						table: table.clone(),
						name,
						index: None,
					})
					.collect();
				Ok(labels)
			} else {
				fetch_columns(&**self.get_database(&table.database)?, table).await
			}
		}
	}
	#[async_recursion(?Send)]
	pub async fn get_view_columns(
		&self,
		view_name: &str,
		database: &Option<String>,
	) -> Result<Option<Vec<String>>> {
		let query = self.get_view_query(view_name, database).await?;
		if let Some(query) = query {
			let plan = Plan::new(self, query, vec![]).await?;
			Ok(Some(plan.labels))
		} else {
			Ok(None)
		}
	}
}
