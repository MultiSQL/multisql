use {
    super::{err_into, SledStorage},
    crate::{Result, Row, Schema, StoreMut, Value},
    async_trait::async_trait,
    sled::{transaction::ConflictableTransactionError, IVec},
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
        let tree = &self.tree;

        for item in tree.scan_prefix(prefix.as_bytes()) {
            let (key, _) = item.map_err(err_into)?;

            tree.remove(key).map_err(err_into)?;
        }

        let key = format!("schema/{}", table_name);
        tree.remove(key).map_err(err_into)?;

        Ok(())
    }

    async fn insert_data(&mut self, table_name: &str, rows: Vec<Row>) -> Result<()> {
        self.tree
            .transaction(|tree| {
                for row in rows.iter() {
                    let id = tree.generate_id()?;
                    let id = id.to_be_bytes();
                    let prefix = format!("data/{}/", table_name);

                    let bytes = prefix
                        .into_bytes()
                        .into_iter()
                        .chain(id.iter().copied())
                        .collect::<Vec<_>>();

                    let key = IVec::from(bytes);
                    let value = bincode::serialize(row)
                        .map_err(err_into)
                        .map_err(ConflictableTransactionError::Abort)?;

                    tree.insert(key, value)?;
                }

                Ok(())
            })
            .map_err(|error| error.into())
    }

    async fn update_data(&mut self, rows: Vec<(Value, Row)>) -> Result<()> {
        self.tree
            .transaction(|tree| {
                for (key, row) in rows.iter() {
                    let value = bincode::serialize(row)
                        .map_err(err_into)
                        .map_err(ConflictableTransactionError::Abort)?;

                    tree.insert(IVec::from(key), value)?;
                }

                Ok(())
            })
            .map_err(|error| error.into())
    }

    async fn delete_data(&mut self, keys: Vec<Value>) -> Result<()> {
        self.tree
            .transaction(|tree| {
                for key in keys.iter() {
                    tree.remove(IVec::from(key))?;
                }

                Ok(())
            })
            .map_err(|error| error.into())
    }
}
