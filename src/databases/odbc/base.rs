use {
	crate::{Column, DBBase, ODBCDatabase, Result, Schema, ValueType},
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
		let mut columns = connection.columns(&connection.current_catalog()?, "", table_name, "")?;
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
