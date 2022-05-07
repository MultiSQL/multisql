use {
	super::base::convert_table_name,
	crate::{Cast, DBMut, ODBCDatabase, Result, Row, Value},
	async_trait::async_trait,
	odbc_api::buffers::TextRowSet,
};

#[async_trait(?Send)]
impl DBMut for ODBCDatabase {
	async fn insert_data(&mut self, table_name: &str, rows: Vec<Row>) -> Result<()> {
		let connection = self
			.environment
			.connect_with_connection_string(&self.connection_string)?;
		let table_name = convert_table_name(table_name);
		let buffer = TextRowSet::for_cursor(
			4096,
			&connection
				.execute(
					&format!("SELECT TOP 1 * FROM {table}", table = table_name),
					(),
				)?
				.unwrap(),
			Some(4096),
		)?;
		let rows = rows
			.into_iter()
			.map(|row| {
				row.0
					.into_iter()
					.map(|cell| {
						if matches!(cell, Value::Null) {
							None
						} else {
							let string: String = cell.cast().unwrap(); // TODO: Handle
							Some(string.as_bytes())
						}
					})
					.collect::<Vec<Option<&[u8]>>>()
					.as_slice()
			})
			.collect::<Vec<&[Option<&[u8]>]>>()
			.as_slice();
		let buffer = create_buffer(buffer, rows);
		connection.execute(
			&format!("INSERT INTO {table} VALUES ?", table = table_name),
			&buffer,
		)?;
		Ok(())
	}
}

fn create_buffer(mut buffer: TextRowSet, rows: &[&[Option<&[u8]>]]) -> TextRowSet {
	// Massive mess. This interface gave me a lot of trouble. TODO: Clean/rewrite
	rows.into_iter().for_each(|row| {
		let row: Vec<Option<&[u8]>> = row.into_iter().map(|cell| *cell).collect();
		buffer.append(row.into_iter());
	});
	buffer
}
