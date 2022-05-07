use {
	crate::{DBBase, ODBCDatabase, Result, Schema},
	async_trait::async_trait,
	odbc_api::Cursor,
};

#[async_trait(?Send)]
impl DBBase for ODBCDatabase {
	async fn scan_schemas(&self) -> Result<Vec<Schema>> {
		let connection = self
			.environment
			.connect_with_connection_string(&self.connection_string)?;
		let mut tables = connection.tables(&connection.current_catalog()?, "", "", "")?;
		let mut output = Vec::new();
		tables.next_row()?.unwrap().get_text(1, &mut output)?;
		println!("{}", std::str::from_utf8(&output).unwrap());
		Ok(Vec::new())
	}
	async fn fetch_schema(&self, _table_name: &str) -> Result<Option<Schema>> {
		Ok(Some(self.scan_schemas().await?.remove(0)))
	}
}
