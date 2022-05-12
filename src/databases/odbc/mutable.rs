use {
	super::base::convert_table_name,
	crate::{DBBase, DBMut, ODBCDatabase, Result, Row, Value},
	async_trait::async_trait,
	odbc_api::{parameter::InputParameter, Bit, IntoParameter},
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
			.collect::<Vec<&str>>();
		for row in rows {
			let (values, columns): (Vec<_>, Vec<&str>) = row
				.0
				.into_iter()
				.enumerate()
				.filter_map(|(index, value)| {
					if !matches!(value, Value::Null) {
						Some((into_parameter(value), columns[index]))
					} else {
						None
					}
				})
				.collect::<Vec<(Box<dyn InputParameter>, &str)>>()
				.into_iter().unzip();
			let placeholders = columns.iter().map(|_|"?").collect::<Vec<&str>>().join(", ");
			let columns = columns.join(", ");
			let query = format!(
					"INSERT INTO {table} ({columns}) VALUES ({placeholders})",
					table = table_name,
					columns = columns
				);
			connection.execute(&query, &values[..]).unwrap();
		}
		Ok(())
	}
}

fn into_parameter(value: Value) -> Box<dyn InputParameter> {
	match value {
		Value::Str(val) => Box::new(val.into_parameter()),
		Value::I64(val) => Box::new(val.into_parameter()),
		Value::U64(val) => Box::new((val as i64).into_parameter()),
		Value::F64(val) => Box::new(val.into_parameter()),
		Value::Bool(val) => Box::new(Bit(val.into())),
		Value::Null => {
			let none: Option<i8> = None;
			Box::new(none.into_parameter())
		}
		_ => unimplemented!(),
	}
}
