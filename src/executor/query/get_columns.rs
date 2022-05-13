use {
	super::select::{refine_items, Manual, Plan},
	crate::{
		executor::{
			fetch::fetch_columns,
			types::{ColumnInfo, ComplexTableName},
		},
		Context, Glue, Result,
	},
	async_recursion::async_recursion,
};

impl Glue {
	pub async fn get_columns(&self, table: ComplexTableName) -> Result<Vec<ColumnInfo>> {
		let context_tables = {
			let context = self.get_context().unwrap();
			context.tables.clone()
		};
		if let Some((context_table_labels, ..)) = context_tables.get(&table.name) {
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
			let plan = Manual::new(self, query)?;
			let (_, columns) = self.arrange_joins(plan.joins).await?;
			let labels = refine_items(plan.select_items, &columns, false)?
				.into_iter()
				.map(|(_recipe, label)| label)
				.collect();
			Ok(Some(labels))
		} else {
			Ok(None)
		}
	}
}
