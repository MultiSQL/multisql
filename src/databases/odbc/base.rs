use {
	crate::{DBBase, ODBCDatabase, Result, Schema},
	async_trait::async_trait,
<<<<<<< HEAD
	odbc_api::{Cursor, ResultSetMetadata},
=======
	odbc_api::Cursor,
>>>>>>> 6301ee113a217d2dd8740cdee1fff688cd488659
};

#[async_trait(?Send)]
impl DBBase for ODBCDatabase {
	async fn scan_schemas(&self) -> Result<Vec<Schema>> {
		let connection = self
			.environment
			.connect_with_connection_string(&self.connection_string)?;
		let mut tables = connection.tables(&connection.current_catalog()?, "", "", "")?;
<<<<<<< HEAD
		let col_range = (0..tables.num_result_cols()?);
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
=======
		let mut output = Vec::new();
		tables.next_row()?.unwrap().get_text(1, &mut output)?;
		println!("{}", std::str::from_utf8(&output).unwrap());
		Ok(Vec::new())
>>>>>>> 6301ee113a217d2dd8740cdee1fff688cd488659
	}
	async fn fetch_schema(&self, _table_name: &str) -> Result<Option<Schema>> {
		Ok(Some(self.scan_schemas().await?.remove(0)))
	}
}
