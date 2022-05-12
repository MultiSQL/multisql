use {
	super::base::convert_table_name,
	crate::{Cast, Column, DBBase, DBMut, ODBCDatabase, Result, Row, Value},
	async_trait::async_trait,
	odbc_api::{
		buffers::{AnyColumnBuffer, ColumnarBuffer, TextColumn},
		parameter::InputParameter,
		Bit, IntoParameter,
	},
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

		let mut insert_columns: Vec<Vec<Value>> = columns.iter().map(|_| Vec::new()).collect();
		for Row(row) in rows {
			for (index, value) in row.into_iter().enumerate() {
				insert_columns[index].push(value);
			}
		}
		let insert_columns: Vec<(usize, Vec<Value>)> = insert_columns
			.into_iter()
			.enumerate()
			.filter(|(_, column)| column.iter().any(|value| !matches!(value, Value::Null)))
			.collect();

		let columns = insert_columns
			.iter()
			.map(|(index, _)| columns[*index].clone())
			.collect::<Vec<&str>>()
			.join(", ");
		let placeholders = insert_columns
			.iter()
			.map(|_| "?")
			.collect::<Vec<&str>>()
			.join(", ");
		let insert_columns: Vec<(u16, AnyColumnBuffer)> = insert_columns
			.into_iter()
			.map(|(index, column)| {
				(
					index as u16,
					into_buffer(column, schema.column_defs[index].clone()),
				)
			})
			.collect(); // TODO: Handle overflow

		let insert_columns = ColumnarBuffer::new(insert_columns);

		let query = format!(
			"INSERT INTO {table} ({columns}) VALUES ({placeholders})",
			table = table_name,
			columns = columns,
			placeholders = placeholders
		);
		connection.execute(&query, &insert_columns).unwrap();
		Ok(())
	}
}

fn into_buffer(values: Vec<Value>, column_def: Column) -> AnyColumnBuffer {
	use crate::ValueType::*;
	match column_def.data_type {
		Timestamp | U64 | I64 => AnyColumnBuffer::I64(
			values
				.into_iter()
				.map(|value| value.cast().unwrap())
				.collect(),
		),
		F64 => AnyColumnBuffer::F64(
			values
				.into_iter()
				.map(|value| value.cast().unwrap())
				.collect(),
		),
		Str => {
			let mut col = TextColumn::new(255, values.len() * 2);
			values.into_iter().enumerate().for_each(|(index, value)| {
				let text: String = value.cast().unwrap();
				col.append(index, Some(text.as_bytes()))
			});
			AnyColumnBuffer::Text(col)
		}
		_ => unimplemented!(),
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
