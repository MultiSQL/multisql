use {
	super::{base::convert_table_name, ColumnSet},
	crate::{Cast, Column, DBBase, DBMut, ODBCDatabase, Result, Row, Value},
	async_trait::async_trait,
	odbc_api::buffers::{AnyColumnBuffer, ColumnarBuffer},
};

const BATCH_SIZE: usize = 1024;

#[async_trait(?Send)]
impl DBMut for ODBCDatabase {
	async fn insert_data(&mut self, table_name: &str, rows: Vec<Row>) -> Result<()> {
		for rows in rows.chunks(255) {
			self.insert(table_name, rows.to_vec()).await?;
		}
		Ok(())
	}
}

impl ODBCDatabase {
	async fn insert(&mut self, table_name: &str, rows: Vec<Row>) -> Result<()> {
		let connection = self
			.environment
			.connect_with_connection_string(&self.connection_string)?;
		let schema = self.fetch_schema(&table_name).await?.unwrap();
		let table_name = convert_table_name(table_name);
		let columns = schema
			.column_defs
			.iter()
			.map(|col_def| col_def.name.as_str())
			.collect::<Vec<&str>>();

		let rows = rows.into_iter().map(|Row(row)| row).collect();
		let column_set = ColumnSet::new(rows, BATCH_SIZE);
		let query = column_set.query(&table_name, &columns);
		let buffers: ColumnarBuffer<AnyColumnBuffer> = column_set.try_into()?;

		connection.execute(&query, &buffers).unwrap();
		Ok(())
	}
}
