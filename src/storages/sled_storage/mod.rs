mod alter_table;
mod auto_increment;
mod error;
mod store;
mod store_mut;
mod util;
#[cfg(not(feature = "alter-table"))]
impl crate::AlterTable for SledStorage {}
#[cfg(not(feature = "auto-increment"))]
impl crate::AutoIncrement for SledStorage {}

use {
    crate::{Error, FullStorage, Result, Schema, Storage},
    error::err_into,
    sled::{self, Config, Db},
    std::convert::TryFrom,
};

#[derive(Debug, Clone)]
pub struct SledStorage {
    tree: Db,
}
impl FullStorage for SledStorage {}
impl SledStorage {
    pub fn new(filename: &str) -> Result<Self> {
        let tree = sled::open(filename).map_err(err_into)?;
        Ok(Self { tree })
    }
}

impl Storage {
    pub fn new_sled(sled: SledStorage) -> Self {
        Self::new(Box::new(sled))
    }
}

impl TryFrom<Config> for SledStorage {
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
