use std::cmp::Ordering;

use rayon::slice::ParallelSliceMut;

use crate::{IndexFilter, Row, Value, join_iters, JoinType, NullOrd};

use {
	super::{err_into, fetch_schema, SledStorage, store_mut::{index_prefix, indexed_key}},
	crate::{Result, RowIter, Schema, Store},
	async_trait::async_trait,
	std::convert::Into,
};

#[async_trait(?Send)]
impl Store for SledStorage {
	async fn fetch_schema(&self, table_name: &str) -> Result<Option<Schema>> {
		fetch_schema(&self.tree, table_name).map(|(_, schema)| schema)
	}

	async fn scan_data(&self, table_name: &str) -> Result<RowIter> {
		let prefix = format!("data/{}/", table_name);

		let result_set = self.tree.scan_prefix(prefix.as_bytes()).map(|item| {
			let (key, value) = item.map_err(err_into)?;
			let value = bincode::deserialize(&value).map_err(err_into)?;

			Ok(((&key).into(), value))
		});

		Ok(Box::new(result_set))
	}

	async fn scan_data_indexed(&self, table_name: &str, index_filters: &[IndexFilter]) -> Result<RowIter> {
		if index_filters.is_empty() {
			self.scan_data(table_name).await
		} else { // TODO: This might be very wrong
			// Only do first filter for now. TODO: More
			let (min_key, max_key) = match index_filters[0].clone() {
				IndexFilter::Between(index_name, min, max) => {
					let prefix = index_prefix(table_name, &index_name);
					let min_key = indexed_key(&prefix, &min)?;
					let max_key = indexed_key(&prefix, &max)?;

					(min_key, max_key)
				}
				_ => unimplemented!()
			};
			let index_results = self.tree.range(min_key..max_key);
			let row_results = index_results.map(|result| {
				result.and_then(|(_index_key, row_key)| self.tree.get(&row_key).map(|row| (row_key, row.unwrap()/*TODO: Handle!*/)))
			});
			let result_set = row_results.map(|item| {
				let (key, value) = item.map_err(err_into)?;
				let value = bincode::deserialize(&value).map_err(err_into)?;

				Ok(((&key).into(), value))
			}).collect::<Vec<Result<(Value, Row)>>>().into_iter(); // Need to collect because of usage of self

			Ok(Box::new(result_set))
		}
	}

	async fn scan_index(&self, table_name: &str, index_filter: IndexFilter) -> Result<Box<dyn Iterator<Item = (Value, Value)>>> {
		use IndexFilter::*;
		match index_filter {
			Between(index_name, min, max) => {
				// TODO: Genericise and optimise
				let prefix = index_prefix(table_name, &index_name);
				let min_key = indexed_key(&prefix, &min)?;
				let max_key = indexed_key(&prefix, &max)?;

				let index_results = self.tree.range(min_key..max_key);
				let mut index_results = index_results.map(|item| {
					let (key, value) = item.map_err(err_into)?;
					let value = bincode::deserialize(&value).map_err(err_into)?;

					Ok(((&key).into(), value))
				}).collect::<Result<Vec<(Value, Value)>>>()?;

				index_results.par_sort_unstable_by(|(_, a), (_, b)| a.null_cmp(b).unwrap_or(Ordering::Equal));
				let index_results = index_results.into_iter().map(|(ik, pk)| (pk, ik));

				Ok(Box::new(index_results))
			}
			Inner(left, right) => {
				let (left, right) = (self.scan_index(table_name, *left).await?, self.scan_index(table_name, *right).await?);
				Ok(Box::new(join_iters(JoinType::Inner, left, right)))
			}
			Outer(left, right) => {
				let (left, right) = (self.scan_index(table_name, *left).await?, self.scan_index(table_name, *right).await?);
				Ok(Box::new(join_iters(JoinType::Outer, left, right)))
			}
		}
	}
}
