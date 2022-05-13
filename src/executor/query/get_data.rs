use {
	crate::{executor::types::LabelsAndRows, Cast, Glue, Result, VIEW_TABLE_NAME},
	sqlparser::ast::Select,
};

impl Glue {
	pub async fn get_view_data(
		&self,
		view_name: &str,
		database: &Option<String>,
	) -> Result<Option<LabelsAndRows>> {
		if let Some(query) = self.get_view_query(view_name, database).await? {
			self.select_query(query, vec![]).await.map(Some)
		} else {
			Ok(None)
		}
	}
	pub async fn get_view_query(
		&self,
		view_name: &str,
		database: &Option<String>,
	) -> Result<Option<Select>> {
		let views = self.get_table_rows(VIEW_TABLE_NAME, database, &None).await;
		if let Ok(views) = views {
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
				Ok(Some(query))
			} else {
				Ok(None)
			}
		} else {
			Ok(None)
		}
	}
}
