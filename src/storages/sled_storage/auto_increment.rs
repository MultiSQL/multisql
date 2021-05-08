#![cfg(feature = "auto-increment")]
use {
	super::{error::err_into, SledStorage},
	crate::{AutoIncrement, Error, Result},
	async_trait::async_trait,
	fstrings::*,
	sled::transaction::ConflictableTransactionError,
};

#[async_trait(?Send)]
impl AutoIncrement for SledStorage {
	async fn generate_increment_values(
		&mut self,
		table_name: String,
		columns: Vec<(
			usize,  /*index*/
			String, /*name*/
			i64,    /*row_count*/
		) /*column*/>,
	) -> Result<
		Vec<(
			(usize /*index*/, String /*name*/), /*column*/
			i64,                                /*start_value*/
		)>,
	> {
		self.tree
			.transaction(|tree| {
				let mut results = vec![];
				for (column_index, column_name, row_count) in &columns {
					// KG: I couldn't get the colunns variable in here for some reason (because it is an enclosure?)
					let (column_index, column_name, row_count): (usize, String, i64) =
						(*column_index, column_name.clone(), *row_count);
					let table_name = table_name.clone();
					let key = f!("generator/{table_name}/{column_name}");
					let key = key.as_bytes();

					let start_ivec = tree.get(key)?;
					let start_value = start_ivec
						.map(|value| bincode::deserialize(&value))
						.unwrap_or(Ok(1))
						.map_err(err_into)
						.map_err(ConflictableTransactionError::Abort)?;

					let end_value = start_value + row_count;
					let end_ivec = bincode::serialize(&end_value)
						.map_err(err_into)
						.map_err(ConflictableTransactionError::Abort)?;

					tree.insert(key, end_ivec)?;
					results.push(((column_index, column_name), start_value));
				}
				Ok(results)
			})
			.map_err(Error::from)
	}

	async fn set_increment_value(
		&mut self,
		table_name: &str,
		column_name: &str,
		end: i64,
	) -> Result<()> {
		let end_ivec = bincode::serialize(&end).map_err(err_into)?;

		let key = f!("generator/{table_name}/{column_name}");
		let key = key.as_bytes();

		self.tree.insert(key, end_ivec).map_err(err_into)?;

		Ok(())
	}
}
