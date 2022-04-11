use std::collections::{BTreeMap, HashMap};

use crate::{join_iters, JoinType, Row};

use {
	crate::{
		DBBase, IndexFilter, MemoryDatabase, MemoryDatabaseError, Plane, Result, Schema, Value,
	},
	async_trait::async_trait,
};

#[async_trait(?Send)]
impl DBBase for MemoryDatabase {
	async fn fetch_schema(&self, table_name: &str) -> Result<Option<Schema>> {
		Ok(self.tables.get(&table_name.to_string()).cloned())
	}
	async fn scan_schemas(&self) -> Result<Vec<Schema>> {
		Ok(self.tables.values().cloned().collect())
	}

	async fn scan_data(&self, table_name: &str) -> Result<Plane> {
		self.data
			.get(&table_name.to_string())
			.cloned()
			.ok_or(MemoryDatabaseError::TableNotFound.into())
			.map(|rows| rows.into_iter().collect())
	}

	async fn scan_data_indexed(
		&self,
		table_name: &str,
		index_filter: IndexFilter,
	) -> Result<Plane> {
		let index_results = self.scan_index(table_name, index_filter).await?;
		let default = HashMap::new();
		let rows = self.data.get(&table_name.to_string()).unwrap_or(&default);
		Ok(index_results
			.into_iter()
			.filter_map(|pk| rows.get(&pk).map(|row| (pk.clone(), row.clone())))
			.collect::<Vec<(Value, Row)>>())
	}

	async fn scan_index(&self, table_name: &str, index_filter: IndexFilter) -> Result<Vec<Value>> {
		use IndexFilter::*;
		match index_filter.clone() {
			LessThan(index_name, ..) | MoreThan(index_name, ..) => {
				let default = BTreeMap::new();
				let index = self
					.indexes
					.get(&table_name.to_string())
					.and_then(|indexes| indexes.get(&index_name))
					.unwrap_or(&default);
				let index_results = match index_filter {
					LessThan(_, max) => index.range(..max),
					MoreThan(_, min) => index.range(min..),
					_ => unreachable!(),
				}
				.map(|(_, pk)| pk.clone())
				.collect();
				Ok(index_results)
			}
			Inner(left, right) => {
				let (left, right) = (
					self.scan_index(table_name, *left),
					self.scan_index(table_name, *right),
				);
				let (left, right) = (left.await?, right.await?);
				Ok(join_iters(JoinType::Inner, left, right))
			}
			Outer(left, right) => {
				let (left, right) = (
					self.scan_index(table_name, *left),
					self.scan_index(table_name, *right),
				);
				let (left, right) = (left.await?, right.await?);
				Ok(join_iters(JoinType::Outer, left, right))
			}
		}
	}
}
