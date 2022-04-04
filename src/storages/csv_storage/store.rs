use {
	super::{utils::csv_reader, CSVStorage},
	crate::{Plane, Result, Row, Schema, Store, Value, WIPError},
	async_trait::async_trait,
};

#[async_trait(?Send)]
impl Store for CSVStorage {
	async fn fetch_schema(&self, _table_name: &str) -> Result<Option<Schema>> {
		Ok(self.schema.clone())
	}

	async fn scan_data(&self, _table_name: &str) -> Result<Plane> {
		let mut reader = csv_reader(&self)?;

		reader
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
			.collect::<Result<_>>()
	}
}
