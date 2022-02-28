use {
	super::{CSVStorage, CSVStorageError},
	crate::{Cast, Result, Row, Schema, StoreMut, WIPError},
	async_trait::async_trait,
	csv::WriterBuilder,
	std::{fs::OpenOptions, io::Write},
};

#[async_trait(?Send)]
impl StoreMut for CSVStorage {
	async fn insert_schema(&mut self, schema: &Schema) -> Result<()> {
		if self.schema.is_some() {
			return Err(CSVStorageError::OnlyOneTableAllowed.into());
		}

		let mut writer = WriterBuilder::new()
			.delimiter(self.csv_settings.delimiter)
			.from_writer(vec![]); // Not good but was having Size issues with moving this elsewhere

		let header: Vec<String> = schema
			.column_defs
			.iter()
			.map(|column_def| column_def.name.value.clone())
			.collect();

		writer
			.write_record(header)
			.map_err(|error| WIPError::Debug(format!("{:?}", error)))?;

		writer
			.flush()
			.map_err(|error| WIPError::Debug(format!("{:?}", error)))?;

		let csv_bytes = writer
			.into_inner()
			.map_err(|error| WIPError::Debug(format!("{:?}", error)))?;

		let mut file = OpenOptions::new()
			.truncate(true)
			.write(true)
			.open(self.path.as_str())
			.map_err(|error| WIPError::Debug(format!("{:?}", error)))?;

		file.write_all(&csv_bytes)
			.map_err(|error| WIPError::Debug(format!("{:?}", error)))?;
		file.flush()
			.map_err(|error| WIPError::Debug(format!("{:?}", error)))?;

		self.schema = Some(schema.clone());
		Ok(())
	}

	async fn delete_schema(&mut self, _table_name: &str) -> Result<()> {
		self.schema = None;
		Ok(())
	}

	async fn insert_data(&mut self, _table_name: &str, rows: Vec<Row>) -> Result<()> {
		let mut writer = WriterBuilder::new()
			.delimiter(self.csv_settings.delimiter)
			.from_writer(vec![]); // Not good but was having Size issues with moving this elsewhere

		for row in rows.into_iter() {
			let string_row = row
				.0
				.into_iter()
				.map(|cell| cell.cast())
				.collect::<Result<Vec<String>>>()?;
			writer
				.write_record(string_row)
				.map_err(|error| WIPError::Debug(format!("{:?}", error)))?;
		}

		writer
			.flush()
			.map_err(|error| WIPError::Debug(format!("{:?}", error)))?;

		let csv_bytes = writer
			.into_inner()
			.map_err(|error| WIPError::Debug(format!("{:?}", error)))?;

		let mut file = OpenOptions::new()
			.append(true)
			.open(self.path.as_str())
			.map_err(|error| WIPError::Debug(format!("{:?}", error)))?;

		file.write_all(&csv_bytes)
			.map_err(|error| WIPError::Debug(format!("{:?}", error)))?;
		file.flush()
			.map_err(|error| WIPError::Debug(format!("{:?}", error)))?;
		Ok(())
	}

	//async fn update_data(&mut self, rows: Vec<(Value, Row)>) -> Result<()>;

	//async fn delete_data(&mut self, keys: Vec<Value>) -> Result<()>;
}
