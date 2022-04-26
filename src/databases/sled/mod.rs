mod auto_increment;
mod base;
mod error;
mod mutable;
mod util;
#[cfg(not(feature = "alter-table"))]
impl crate::AlterTable for SledDatabase {}
#[cfg(not(feature = "auto-increment"))]
impl crate::AutoIncrement for SledDatabase {}

use {
	crate::{DBFull, Database, Error, Result, Schema},
	error::err_into,
	sled::{self, Config, Db},
	std::convert::TryFrom,
};

#[derive(Debug, Clone)]
pub struct SledDatabase {
	tree: Db,
}
impl DBFull for SledDatabase {}
impl SledDatabase {
	pub fn new(filename: &str) -> Result<Self> {
		let tree = sled::open(filename).map_err(err_into)?;
		Ok(Self { tree })
	}
}

impl Database {
	pub fn new_sled(sled: SledDatabase) -> Self {
		Self::new(Box::new(sled))
	}
}

impl TryFrom<Config> for SledDatabase {
	type Error = Error;

	fn try_from(config: Config) -> Result<Self> {
		let tree = config.open().map_err(err_into)?;

		Ok(Self { tree })
	}
}

fn fetch_schema(tree: &Db, table_name: &str) -> Result<(String, Option<Schema>)> {
	let key = format!("schema/{}", table_name);
	let value = tree.get(&key.as_bytes()).map_err(err_into)?;
	let schema = value
		.map(|v| bincode::deserialize(&v))
		.transpose()
		.map_err(err_into)?;

	Ok((key, schema))
}
