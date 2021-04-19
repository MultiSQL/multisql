use {
    super::{auto_increment, columns_to_positions, validate, validate_unique},
    crate::{
        data::{get_name, Schema},
        executor::{query::query, types::Row as VecRow},
        macros::{try_block, try_into},
        result::MutResult,
        store::{AlterTable, AutoIncrement, Store, StoreMut},
        ExecuteError, Payload, Row,
    },
    sqlparser::ast::{ColumnDef, Ident, ObjectName, Query},
    std::fmt::Debug,
};

pub async fn insert<
    Key: 'static + Debug,
    Storage: Store<Key> + StoreMut<Key> + AlterTable + AutoIncrement,
>(
    storage: Storage,
    table_name: &ObjectName,
    columns: &Vec<Ident>,
    source: &Box<Query>,
) -> MutResult<Storage, Payload> {
    let (rows, table_name, column_defs): (Vec<VecRow>, &String, Vec<ColumnDef>) =
        try_block!(storage, {
            let table_name = get_name(table_name)?;
            let Schema { column_defs, .. } = storage
                .fetch_schema(table_name)
                .await?
                .ok_or(ExecuteError::TableNotExists)?;

            let (_, rows) = query(&storage, *source.clone()).await?;

            let column_positions = columns_to_positions(&column_defs, columns)?;

            let rows = validate(&column_defs, &column_positions, rows)?;

            Ok((rows, table_name, column_defs))
        });

    #[cfg(feature = "auto-increment")]
    let (storage, rows) = auto_increment(storage, table_name, &column_defs, rows).await?;
    try_into!(
        storage,
        validate_unique(&storage, table_name, &column_defs, &rows).await
    );
    let rows: Vec<Row> = rows.into_iter().map(Row).collect();

    let num_rows = rows.len();

    storage
        .insert_data(table_name, rows)
        .await
        .map(|(storage, _)| (storage, Payload::Insert(num_rows)))
}
