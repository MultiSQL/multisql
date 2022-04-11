use {
	super::{err_into, fetch_schema, SledStorage},
	crate::{
		BigEndian, Column, Result, Row, Schema, SchemaChange, SchemaDiff, StorageError, StoreMut,
		Value,
	},
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

	async fn update_data(&mut self, _table_name: &str, rows: Vec<(Value, Row)>) -> Result<()> {
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

	async fn delete_data(&mut self, _table_name: &str, keys: Vec<Value>) -> Result<()> {
		let batch = keys
			.into_iter()
			.fold(sled::Batch::default(), |mut batch, key| {
				batch.remove(IVec::from(&key));
				batch
			});
		self.tree.apply_batch(batch).map_err(err_into)
	}

	async fn update_index(
		&mut self,
		table_name: &str,
		index_name: &str,
		keys: Vec<(Value, Value)>,
	) -> Result<()> {
		let prefix = index_prefix(table_name, index_name);

		let remove_keys = self
			.tree
			.scan_prefix(prefix.as_bytes())
			.par_bridge()
			.map(|result| result.map(|(key, _)| key).map_err(err_into))
			.collect::<Result<Vec<_>>>()?;

		let keys: Vec<(IVec, IVec)> = keys
			.into_iter()
			.enumerate()
			.map(|(idx, (index_key, row_key))| {
				// TODO: Don't use idx where unique
				let index_key = unique_indexed_key(&prefix, &index_key, idx)?;
				let row_key = IVec::from(&row_key);
				Ok((index_key, row_key))
			})
			.collect::<Result<Vec<(IVec, IVec)>>>()?;

		let batch = remove_keys
			.into_iter()
			.fold(sled::Batch::default(), |mut batch, key| {
				batch.remove(key);
				batch
			});
		let batch = keys
			.into_iter()
			.fold(batch, |mut batch, (index_key, row_key)| {
				batch.insert(index_key, row_key);
				batch
			});

		self.tree.apply_batch(batch).map_err(err_into)
	}

	async fn alter_table(&mut self, table_name: &str, schema_diff: SchemaDiff) -> Result<()> {
		let changes = schema_diff.get_changes();
		for change in changes.into_iter() {
			use SchemaChange::*;
			match change {
				RenameTable(new_name) => self.rename_table(table_name, new_name),
				ColumnAdd(column_def) => self.add_column(table_name, column_def),
				ColumnRemove(index) => self.remove_column(table_name, index),
				ColumnUpdate(..) | IndexAdd(..) => Ok(()),
				_ => Err(StorageError::Unimplemented.into()),
				// TODO: Column remove & add: manipulate all rows
				// TODO: Index remove, add and update: rebuild
			}?;
		}

		let (key, schema) = fetch_schema(&self.tree, table_name)?;
		let schema = schema.ok_or(StorageError::TableNotFound)?;
		let schema = schema_diff.merge(schema);
		let schema_value = bincode::serialize(&schema).map_err(err_into)?;
		self.tree.insert(key, schema_value).map_err(err_into)?;

		Ok(())
	}
}

impl SledStorage {
	pub fn rename_table(&mut self, old_name: &str, new_name: String) -> Result<()> {
		let (key, schema) = fetch_schema(&self.tree, old_name)?;
		let schema = schema.ok_or(StorageError::TableNotFound)?;
		self.tree.remove(key).map_err(err_into)?;

		let value = bincode::serialize(&schema).map_err(err_into)?;
		let key = format!("schema/{}", new_name);
		let key = key.as_bytes();
		self.tree.insert(key, value).map_err(err_into)?;

		let prefix = format!("data/{}/", old_name);

		for item in self.tree.scan_prefix(prefix.as_bytes()) {
			let (key, value) = item.map_err(err_into)?;

			let new_key = std::str::from_utf8(key.as_ref()).map_err(err_into)?;
			let new_key = new_key.replace(old_name, &new_name);
			self.tree.insert(new_key, value).map_err(err_into)?;

			self.tree.remove(key).map_err(err_into)?;
		}

		Ok(())
	}
	pub fn add_column(&mut self, table_name: &str, column: Column) -> Result<()> {
		let value = match (&column.default, &column.is_nullable) {
			(Some(_expr), _) => Err(StorageError::Unimplemented), // TODO
			(None, true) => Ok(Value::Null),
			(None, false) => Err(StorageError::Unimplemented),
		}?;

		let prefix = format!("data/{}/", table_name);

		for item in self.tree.scan_prefix(prefix.as_bytes()) {
			let (key, row) = item.map_err(err_into)?;
			let row: Row = bincode::deserialize(&row).map_err(err_into)?;
			let row = Row(row.0.into_iter().chain([value.clone()]).collect());
			let row = bincode::serialize(&row).map_err(err_into)?;

			self.tree.insert(key, row).map_err(err_into)?;
		}
		Ok(())
	}
	pub fn remove_column(&mut self, table_name: &str, index: usize) -> Result<()> {
		let prefix = format!("data/{}/", table_name);
		for item in self.tree.scan_prefix(prefix.as_bytes()) {
			let (key, row) = item.map_err(err_into)?;
			let mut row = bincode::deserialize(&row).map_err(err_into)?.0;
			row.remove(index);
			let row = bincode::serialize(&Row(row)).map_err(err_into)?;

			self.tree.insert(key, row).map_err(err_into)?;
		}
		Ok(())
	}
}

pub fn index_prefix(table_name: &str, index_name: &str) -> String {
	format!("index/{}/{}/", table_name, index_name)
}

pub fn indexed_key(prefix: &str, index: &Value) -> Result<IVec> {
	Ok([prefix.as_bytes(), &index.to_be_bytes()].concat().into())
}
pub fn unique_indexed_key(prefix: &str, index: &Value, idx: usize) -> Result<IVec> {
	Ok([
		prefix.as_bytes(),
		&index.to_be_bytes(),
		&[0x00],
		&idx.to_be_bytes(),
	]
	.concat()
	.into())
}
