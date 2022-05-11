use {
	super::base::convert_table_name,
	crate::{Cast, DBBase, DBMut, ODBCDatabase, Result, Row},
	async_trait::async_trait,
	odbc_api::IntoParameter,
};

#[async_trait(?Send)]
impl DBMut for ODBCDatabase {
	async fn insert_data(&mut self, table_name: &str, rows: Vec<Row>) -> Result<()> {
		let connection = self
			.environment
			.connect_with_connection_string(&self.connection_string)?;
			
		let schema = self.fetch_schema(&table_name).await?.unwrap();
		let table_name = convert_table_name(table_name);
		let columns = schema
			.column_defs
			.iter()
			.map(|col_def| col_def.name.as_str())
			.collect::<Vec<&str>>()
			.join(", ");
		let placeholders = schema
			.column_defs
			.iter()
			.map(|_| "?")
			.collect::<Vec<&str>>()
			.join(", ");
		let query = format!(
			"INSERT INTO {table} ({columns}) VALUES ({placeholders})",
			table = table_name,
			columns = columns
		);
		for row in rows {
			let values = row
				.0
				.into_iter()
				.map(|value| value.cast().map(|value: String| value.into_parameter()))
				.collect::<Result<Vec<_>>>()?;
			connection.execute(&query, &values[..]).unwrap();
		}
		Ok(())
	}
}
