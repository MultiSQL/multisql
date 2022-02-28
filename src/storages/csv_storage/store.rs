use {
	super::utils::csv_reader,
	super::CSVStorage,
	crate::{Result, Row, RowIter, Schema, Store, Value, WIPError},
	async_trait::async_trait,
};

#[async_trait(?Send)]
impl Store for CSVStorage {
	async fn fetch_schema(&self, _table_name: &str) -> Result<Option<Schema>> {
		Ok(self.schema.clone())
	}

	async fn scan_data(&self, _table_name: &str) -> Result<RowIter> {
		let mut reader = csv_reader(&self)?;

		let keyed_rows: Vec<Result<(Value, Row)>> = reader
			.records()
			.enumerate()
			.map(|(index, record)| {
				record
					.map_err(|error| WIPError::Debug(format!("{:?}", error)).into())
					.map(|record| {
						(
							Value::I64(index as i64),
							Row(record
								.into_iter()
								.map(|cell| Value::Str(cell.to_string()))
								.collect()),
						)
					})
			})
			.collect();

		Ok(Box::new(keyed_rows.into_iter()))
	}
}
