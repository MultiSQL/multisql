use {
	crate::{Cast, Column, DBBase, ODBCDatabase, Plane, Result, Row, Schema, Value, ValueType},
	async_trait::async_trait,
	odbc_api::{Cursor, ResultSetMetadata},
};

#[async_trait(?Send)]
impl DBBase for ODBCDatabase {
	async fn scan_schemas(&self) -> Result<Vec<Schema>> {
		let connection = self
			.environment
			.connect_with_connection_string(&self.connection_string)?;
		let mut tables = connection.tables(&connection.current_catalog()?, "", "", "TABLE")?;
		let col_range = 1..(tables.num_result_cols()?);
		let mut schemas = Vec::new();
		while let Some(mut row) = tables.next_row()? {
			let row = col_range
				.clone()
				.map(|col| {
					let mut output = Vec::new();
					let output = row
						.get_text(col as u16, &mut output)
						.map(|_| std::str::from_utf8(&output).unwrap_or_default())
						.unwrap_or("NULL")
						.to_string();
					Ok(output)
				})
				.collect::<Result<Vec<String>>>()?;
			if let Some(schema) = self.fetch_schema(&row[4]).await? {
				schemas.push(schema);
			}
		}
		Ok(schemas)
	}
	async fn fetch_schema(&self, table_name: &str) -> Result<Option<Schema>> {
		let connection = self
			.environment
			.connect_with_connection_string(&self.connection_string)?;
		let (schema_name, table_name) = convert_table_name(table_name);
		let mut columns =
			connection.columns(&connection.current_catalog()?, schema_name, table_name, "")?;
		let col_range = 1..(columns.num_result_cols()?);
		let mut column_defs = Vec::new();
		while let Some(mut row) = columns.next_row()? {
			let row = col_range
				.clone()
				.map(|col| {
					let mut output = Vec::new();
					let output = row
						.get_text(col as u16, &mut output)
						.map(|_| std::str::from_utf8(&output).unwrap_or_default())
						.unwrap_or("NULL")
						.to_string();
					Ok(output)
				})
				.collect::<Result<Vec<String>>>()?;
			// TODO: Be safer with referencing elements -- doesn't matter that much though
			column_defs.push(Column {
				name: row[3].clone(),
				data_type: odbc_type_to_multisql(&row[5]),
				default: None, // doesn't really matter
				is_nullable: (row[17] != "NO"),
				is_unique: false, // doesn't realllllyyyy matter
			});
		}
		Ok(if !column_defs.is_empty() {
			Some(Schema {
				table_name: table_name.to_string(),
				column_defs,
				indexes: Vec::new(), // TODO
			})
		} else {
			None
		})
	}

	async fn scan_data(&self, table_name: &str) -> Result<Plane> {
		// TODO: Non-string conversion (if possible?)
		let connection = self
			.environment
			.connect_with_connection_string(&self.connection_string)?;

		let schema = self.fetch_schema(table_name).await?.unwrap(); // TODO: Handle
		let (schema_name, table_name) = convert_table_name(table_name);
		let table_name = if !schema_name.is_empty() {
			format!("{}.{}", schema_name, table_name)
		} else {
			table_name.to_string()
		};

		let response =
			connection.execute(&format!("SELECT * FROM {table}", table = table_name), ())?;
		Ok(if let Some(mut rows) = response {
			let col_range = 1..(rows.num_result_cols()?);

			let mut out_rows = Vec::new();
			while let Some(mut row) = rows.next_row()? {
				let row = col_range
					.clone()
					.map(|col| {
						let mut output = Vec::new();
						let output = row
							.get_text(col as u16, &mut output)
							.map(|_| std::str::from_utf8(&output).unwrap_or_default())
							.unwrap_or("NULL")
							.to_string();
						let column = &schema.column_defs[(col - 1) as usize]; // TODO: Protect
						odbc_value_to_multisql(output, &column.data_type)
					})
					.collect::<Result<Vec<Value>>>()?;
				out_rows.push((Value::Null, Row(row))); // TODO: PK
			}
			out_rows
		} else {
			Vec::new()
		})
	}
}

fn convert_table_name(table_name: &str) -> (&str, &str) {
	let mut table_name: Vec<&str> = table_name.split('_').collect();
	if table_name.len() == 1 {
		("", table_name.remove(0))
	} else if table_name.len() == 2 {
		(table_name.remove(0), table_name.remove(0))
	} else {
		("", "")
	}
}

fn odbc_type_to_multisql(data_type: &str) -> ValueType {
	match data_type {
		"bigint" /*lossy*/ | "tinyint" | "smallint" | "int" => ValueType::I64,
		"decimal" /*lossy*/ | "money" /*lossy*/ | "float" => ValueType::F64,
		"smalldatetime" | "datetime" => ValueType::Timestamp,
		"bit" => ValueType::Bool,
		"varchar" => ValueType::Str,
		_ => {
			ValueType::Any
		}
	}
}

fn odbc_value_to_multisql(data_value: String, data_type: &ValueType) -> Result<Value> {
	if data_value == "NULL" || (data_value.is_empty() && !matches!(data_type, ValueType::Str)) {
		return Ok(Value::Null);
	}
	let from = Value::Str(data_value.clone());
	match data_type {
		ValueType::I64 => Ok(from
			.cast()
			.map(Value::I64)
			.expect(&format!("{}", data_value))),
		ValueType::F64 => Ok(from
			.cast()
			.map(Value::F64)
			.expect(&format!("{}", data_value))),
		ValueType::Timestamp => Ok(Value::Null), // TODO
		ValueType::Bool => from
			.cast()
			.map(Value::I64)
			.and_then(|int| int.cast().map(Value::Bool)),
		ValueType::Str => Ok(from),
		_ => Ok(from),
	}
}
