use {
	crate::{DBBase, ODBCDatabase, Result, Schema},
	async_trait::async_trait,
	odbc_api::{Cursor, ResultSetMetadata},
};

#[async_trait(?Send)]
impl DBBase for ODBCDatabase {
	async fn scan_schemas(&self) -> Result<Vec<Schema>> {
		let connection = self
			.environment
			.connect_with_connection_string(&self.connection_string)?;
		let mut tables = connection.tables(&connection.current_catalog()?, "", "", "")?;
		let col_range = 1..(tables.num_result_cols()?);
		let mut schemas = Vec::new();
		while let Some(mut row) = tables.next_row()? {
			let row = col_range
				.clone()
				.map(|col| {
					let mut output = Vec::new();
					row.get_text(col as u16, &mut output)?;
					let output = std::str::from_utf8(&output)?.to_string();
					Ok(output)
				})
				.collect::<Result<Vec<String>>>()?;
			println!("{:?}", row);
		}
		Ok(schemas)
	}
	async fn fetch_schema(&self, _table_name: &str) -> Result<Option<Schema>> {
		Ok(Some(self.scan_schemas().await?.remove(0)))
	}
}
