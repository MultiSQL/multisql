use {
	super::{
		err_into, fetch_schema,
		mutable::{index_prefix, indexed_key},
		SledDatabase,
	},
	crate::{
		join_iters, DBBase, IndexFilter, JoinType, NullOrd, Plane, Result, Row, Schema, Value,
	},
	async_trait::async_trait,
	rayon::slice::ParallelSliceMut,
	sled::IVec,
	std::{cmp::Ordering, convert::Into},
};

#[async_trait(?Send)]
impl DBBase for SledDatabase {
	async fn fetch_schema(&self, table_name: &str) -> Result<Option<Schema>> {
		fetch_schema(&self.tree, table_name).map(|(_, schema)| schema)
	}
	async fn scan_schemas(&self) -> Result<Vec<Schema>> {
		let prefix = "schema/".to_string();
		self.tree
			.scan_prefix(prefix.as_bytes())
			.map(|item| {
				let (_, bytes) = item.map_err(err_into)?;
				bincode::deserialize(&bytes).map_err(err_into)
			})
			.collect()
	}

	async fn scan_data(&self, table_name: &str) -> Result<Plane> {
		let prefix = format!("data/{}/", table_name);

		self.tree
			.scan_prefix(prefix.as_bytes())
			.map(|item| {
				let (key, value) = item.map_err(err_into)?;
				let value = bincode::deserialize(&value).map_err(err_into)?;

				Ok(((&key).into(), value))
			})
			.collect::<Result<Vec<(Value, Row)>>>()
	}

	async fn scan_data_indexed(
		&self,
		table_name: &str,
		index_filter: IndexFilter,
	) -> Result<Plane> {
		let index_results = self.scan_index(table_name, index_filter).await?;
		let row_results = index_results.into_iter().map(|pk| {
			if let Value::Bytes(pk) = pk {
				self.tree
					.get(&pk)
					.map(|row| (pk, row.unwrap() /*TODO: Handle!*/))
			} else {
				unreachable!();
			}
		});
		row_results
			.map(|item| {
				let (pk, value) = item.map_err(err_into)?;
				let value = bincode::deserialize(&value).map_err(err_into)?;

				Ok((Value::Bytes(pk.to_vec()), value))
			})
			.collect::<Result<Vec<(Value, Row)>>>()
	}

	async fn scan_index(&self, table_name: &str, index_filter: IndexFilter) -> Result<Vec<Value>> {
		use IndexFilter::*;
		match index_filter.clone() {
			LessThan(index_name, ..) | MoreThan(index_name, ..) => {
				// TODO: Genericise and optimise
				let prefix = index_prefix(table_name, &index_name);
				let abs_min = IVec::from(prefix.as_bytes());
				let abs_max = IVec::from([prefix.as_bytes(), &[0xFF]].concat());

				let index_results = match index_filter {
					LessThan(_, max) => self.tree.range(abs_min..indexed_key(&prefix, &max)?),
					MoreThan(_, min) => self.tree.range(indexed_key(&prefix, &min)?..abs_max),
					_ => unreachable!(),
				};
				let mut index_results = index_results
					.map(|item| {
						let (_, pk) = item.map_err(err_into)?;
						let pk = Value::Bytes(pk.to_vec());

						Ok(pk)
					})
					.collect::<Result<Vec<Value>>>()?;

				index_results.par_sort_unstable_by(|a, b| a.null_cmp(b).unwrap_or(Ordering::Equal));
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
