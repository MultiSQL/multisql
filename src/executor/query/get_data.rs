use {
	crate::{
		executor::types::LabelsAndRows, Cast, Glue, Result, VIEW_TABLE_NAME,
	},
	async_recursion::async_recursion,
};

impl Glue {
	#[async_recursion(?Send)]
	pub async fn get_view_data(
		&self,
		view_name: &str,
		database: &Option<String>,
	) -> Result<Option<LabelsAndRows>> {
		let views = self
			.get_table_rows(VIEW_TABLE_NAME, database, &None)
			.await?;
		let query = views.into_iter().find_map(|row| {
			let name: String = row[0].clone().cast().unwrap();
			if view_name == name {
				Some(row[1].clone())
			} else {
				None
			}
		});
		if let Some(query) = query {
			let query: String = query.clone().cast()?;
			let query = serde_yaml::from_str(&query).unwrap(); // TODO: Handle
			self.no_cte_query(query).await.map(Some)
		} else {
			Ok(None)
		}
	}
}
