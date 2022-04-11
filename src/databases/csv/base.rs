use {
	super::{utils::csv_reader, CSVDatabase},
	crate::{DBBase, Plane, Result, Row, Schema, Value, WIPError},
	async_trait::async_trait,
};

#[async_trait(?Send)]
impl DBBase for CSVDatabase {
	async fn fetch_schema(&self, _table_name: &str) -> Result<Option<Schema>> {
		Ok(self.schema.clone())
	}
	async fn scan_schemas(&self) -> Result<Vec<Schema>> {
		Ok(self
			.schema
			.clone()
			.map(|schema| vec![schema])
			.unwrap_or_default())
	}

	async fn scan_data(&self, _table_name: &str) -> Result<Plane> {
		let mut reader = csv_reader(self)?;

		#[allow(clippy::needless_collect)]
		// Clippy doesn't understand the need. Needed because we have borrowed values within.
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
