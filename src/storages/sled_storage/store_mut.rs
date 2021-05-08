use {
	super::{err_into, SledStorage},
	crate::{Result, Row, Schema, StoreMut, Value},
	async_trait::async_trait,
	rayon::prelude::*,
	sled::IVec,
	std::convert::From,
};

#[async_trait(?Send)]
impl StoreMut for SledStorage {
	async fn insert_schema(&mut self, schema: &Schema) -> Result<()> {
		let key = format!("schema/{}", schema.table_name);
		let key = key.as_bytes();
		let value = bincode::serialize(schema).map_err(err_into)?;

		self.tree.insert(key, value).map_err(err_into)?;

		Ok(())
	}

	async fn delete_schema(&mut self, table_name: &str) -> Result<()> {
		let prefix = format!("data/{}/", table_name);

		let mut keys = self
			.tree
			.scan_prefix(prefix.as_bytes())
			.par_bridge()
			.map(|result| result.map(|(key, _)| key).map_err(err_into))
			.collect::<Result<Vec<_>>>()?;

		let table_key = format!("schema/{}", table_name);
		keys.push(IVec::from(table_key.as_bytes()));

		let batch = keys
			.into_iter()
			.fold(sled::Batch::default(), |mut batch, key| {
				batch.remove(key);
				batch
			});

		self.tree.apply_batch(batch).map_err(err_into)
	}

	async fn insert_data(&mut self, table_name: &str, rows: Vec<Row>) -> Result<()> {
		let ready_rows = rows
			.into_par_iter()
			.map(|row| {
				let id = self.tree.generate_id().map_err(err_into)?;
				let id = id.to_be_bytes();
				let prefix = format!("data/{}/", table_name);

				let bytes = prefix
					.into_bytes()
					.into_iter()
					.chain(id.iter().copied())
					.collect::<Vec<_>>();

				let key = IVec::from(bytes);
				let value = bincode::serialize(&row).map_err(err_into)?;
				Ok((key, value))
			})
			.collect::<Result<Vec<(_, _)>>>()?;

		let batch =
			ready_rows
				.into_iter()
				.fold(sled::Batch::default(), |mut batch, (key, value)| {
					batch.insert(key, value);
					batch
				});
		self.tree.apply_batch(batch).map_err(err_into)
	}

	async fn update_data(&mut self, rows: Vec<(Value, Row)>) -> Result<()> {
		let ready_rows = rows
			.into_par_iter()
			.map(|(key, value)| {
				let value = bincode::serialize(&value).map_err(err_into)?;
				let key = IVec::from(&key);
				Ok((key, value))
			})
			.collect::<Result<Vec<(_, _)>>>()?;

		let batch =
			ready_rows
				.into_iter()
				.fold(sled::Batch::default(), |mut batch, (key, value)| {
					batch.insert(key, value);
					batch
				});
		self.tree.apply_batch(batch).map_err(err_into)
	}

	async fn delete_data(&mut self, keys: Vec<Value>) -> Result<()> {
		let batch = keys
			.into_iter()
			.fold(sled::Batch::default(), |mut batch, key| {
				batch.remove(IVec::from(&key));
				batch
			});
		self.tree.apply_batch(batch).map_err(err_into)
	}
}
