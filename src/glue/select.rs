use {
	super::Glue,
	crate::{Cast, ExecuteError, Payload, Result},
	serde_json::value::Value as JSONValue,
};

/// ## Select (`SELECT`)
impl Glue {
	/// Only for `SELECT` queries.
	///
	/// Output is one big [serde_json::Value], wrapped in a [Result].
	///
	/// Generally useful for webby interactions.
	pub fn select_json(&mut self, query: &str) -> Result<JSONValue> {
		// TODO: Make this more efficient and not affect database if not select by converting earlier
		if let Payload::Select { labels, rows } = self.execute(query)? {
			let rows = JSONValue::Array(
				rows.into_iter()
					.map(|row| {
						JSONValue::Object(
							row.0
								.into_iter()
								.enumerate()
								.map(|(index, cell)| (labels[index].clone(), cell.into()))
								.collect::<serde_json::map::Map<String, JSONValue>>(),
						)
					})
					.collect(),
			);
			Ok(rows)
		} else {
			Err(ExecuteError::QueryNotSupported.into())
		}
	}

	/// Only for `SELECT` queries.
	pub fn select_as_string(&mut self, query: &str) -> Result<Vec<Vec<String>>> {
		// TODO: Make this more efficient and not affect database
		if let Payload::Select { labels, rows } = self.execute(query)? {
			Ok(vec![labels]
				.into_iter()
				.chain(
					rows.into_iter()
						.map(|row| {
							row.0
								.into_iter()
								.map(|value| value.cast())
								.collect::<Result<Vec<String>>>()
						})
						.collect::<Result<Vec<Vec<String>>>>()?,
				)
				.collect())
		} else {
			Err(ExecuteError::QueryNotSupported.into())
		}
	}

	/// Only for `SELECT` queries.
	pub fn select_as_csv(&mut self, query: &str) -> Result<String> {
		// TODO: Don't use `unwrap()`
		if let Payload::Select { labels, rows } = self.execute(query)? {
			{
				let mut writer = csv::Writer::from_writer(vec![]);
				writer.write_record(labels).unwrap();
				for row in rows.into_iter() {
					for field in row.0.into_iter() {
						let string: String = field.cast()?;
						writer.write_field(string).unwrap();
					}
					writer.write_record(None::<&[u8]>).unwrap();
				}
				let bytes = writer.into_inner().unwrap();
				let string = String::from_utf8(bytes).unwrap();
				Some(string)
			}
			.map(Ok)
			.unwrap()
		} else {
			Err(ExecuteError::QueryNotSupported.into())
		}
	}
}
