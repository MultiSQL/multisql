use {
    super::{JoinMethod, JoinType},
    crate::{
        executor::types::{Row, Table},
        store::Store,
        Result,
    },
    std::fmt::Debug,
};

pub struct JoinExecute {
    pub table: Table,
    pub method: JoinMethod,
    pub join_type: JoinType,
}

impl JoinExecute {
    pub async fn get_rows<'a, Key: 'static + Debug>(
        &self,
        storage: &'a dyn Store<Key>,
    ) -> Result<Vec<Row>> {
        storage
            .scan_data(self.table.as_str())
            .await?
            .map(|result| result.map(|(_, row)| row.0))
            .collect::<Result<Vec<Row>>>()
    }
    pub async fn execute<'a, Key: 'static + Debug>(
        self,
        storage: &'a dyn Store<Key>,
        plane_rows: Vec<Row>,
    ) -> Result<Vec<Row>> {
        let rows = self.get_rows(storage).await?;
        self.method.run(&self.join_type, plane_rows, rows)
    }
}
