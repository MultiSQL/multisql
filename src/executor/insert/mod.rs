mod validate;

pub use validate::{columns_to_positions, validate};
use {
    crate::{
        data::{get_name, Schema},
        executor::query::query,
        result::MutResult,
        store::{AlterTable, AutoIncrement, Store, StoreMut},
        ExecuteError, Payload, Row,
    },
    serde::Serialize,
    sqlparser::ast::{Ident, ObjectName, Query},
    std::fmt::Debug,
    thiserror::Error as ThisError,
};

#[derive(ThisError, Serialize, Debug, PartialEq)]
pub enum InsertError {
    #[error("expected value for column which neither accepts NULL nor has a default")]
    MissingValue,
    #[error("wrong number of values in insert statement")]
    WrongNumberOfValues,
    #[error("default value failed to be calculated")]
    BadDefault,
    #[error("column '{0}' not found")]
    ColumnNotFound(String),
}

macro_rules! try_block {
    ($storage: expr, $block: block) => {{
        match (|| async { $block })().await {
            Err(e) => {
                return Err(($storage, e));
            }
            Ok(v) => v,
        }
    }};
}

pub async fn insert<
    Key: 'static + Debug,
    Storage: Store<Key> + StoreMut<Key> + AlterTable + AutoIncrement,
>(
    storage: Storage,
    table_name: &ObjectName,
    columns: &Vec<Ident>,
    source: &Box<Query>,
) -> MutResult<Storage, Payload> {
    let (rows, table_name): (Vec<Row>, &String) = try_block!(storage, {
        let table_name = get_name(table_name)?;
        let Schema { column_defs, .. } = storage
            .fetch_schema(table_name)
            .await?
            .ok_or(ExecuteError::TableNotExists)?;

        let (_, rows) = query(&storage, *source.clone()).await?;

        let column_positions = columns_to_positions(&column_defs, columns)?;

        let rows = validate(&column_defs, &column_positions, rows)?;

        let rows = rows.into_iter().map(Row).collect(); // I don't like this.

        Ok((rows, table_name))
    });

    let num_rows = rows.len();

    storage
        .insert_data(table_name, rows)
        .await
        .map(|(storage, _)| (storage, Payload::Insert(num_rows)))
}
