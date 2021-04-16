use {
    super::{
        alter::{create_table, drop},
        insert::insert,
        //types::Row,
        /*update::Update,*/
        query::query,
    },
    crate::{
        data::get_name,
        parse_sql::Query,
        result::MutResult,
        store::{AlterTable, AutoIncrement, Store, StoreMut},
        Row,
    },
    serde::Serialize,
    sqlparser::ast::Statement,
    std::fmt::Debug,
    thiserror::Error as ThisError,
};

#[cfg(feature = "alter-table")]
use super::alter::alter_table;

#[derive(ThisError, Serialize, Debug, PartialEq)]
pub enum ExecuteError {
    #[error("query not supported")]
    QueryNotSupported,

    #[error("unsupported insert value type: {0}")]
    UnreachableUnsupportedInsertValueType(String),

    #[error("table does not exist")]
    TableNotExists,
}

#[derive(Serialize, Debug, PartialEq)]
pub enum Payload {
    Create,
    Insert(usize),
    Select {
        labels: Vec<String>,
        rows: Vec<Row>,
    },
    Delete(usize),
    Update(usize),
    DropTable,

    #[cfg(feature = "alter-table")]
    AlterTable,
}

pub async fn execute<
    Key: 'static + Debug,
    Storage: Store<Key> + StoreMut<Key> + AlterTable + AutoIncrement,
>(
    storage: Storage,
    statement: &Query,
) -> MutResult<Storage, Payload> {
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
    macro_rules! try_into {
        ($storage: expr, $expr: expr) => {
            match $expr {
                Err(e) => {
                    return Err(($storage, e));
                }
                Ok(v) => v,
            }
        };
    }

    let Query(statement) = statement;

    match statement {
        //- Modification
        //-- Tables
        Statement::CreateTable {
            name,
            columns,
            if_not_exists,
            ..
        } => create_table(storage, name, columns, *if_not_exists)
            .await
            .map(|(storage, _)| (storage, Payload::Create)),
        Statement::Drop {
            object_type,
            names,
            if_exists,
            ..
        } => drop(storage, object_type, names, *if_exists)
            .await
            .map(|(storage, _)| (storage, Payload::DropTable)),
        #[cfg(feature = "alter-table")]
        Statement::AlterTable { name, operation } => alter_table(storage, name, operation)
            .await
            .map(|(storage, _)| (storage, Payload::AlterTable)),

        //-- Rows
        Statement::Insert {
            table_name,
            columns,
            source,
            ..
        } => insert(storage, table_name, columns, source).await,
        /*Statement::Update {
            table_name,
            selection,
            assignments,
        } => {
            let rows = try_block!(storage, {
                let table_name = get_name(table_name)?;
                let Schema { column_defs, .. } = storage
                    .fetch_schema(table_name)
                    .await?
                    .ok_or(ExecuteError::TableNotExists)?;
                let update = Update::new(&storage, table_name, assignments, &column_defs)?;
                let filter = Filter::new(&storage, selection.as_ref(), None, None);

                let all_columns = Rc::from(update.all_columns());
                let columns_to_update = update.columns_to_update();
                let rows = fetch(&storage, table_name, Rc::clone(&all_columns), filter)
                    .await?
                    .and_then(|item| {
                        let update = &update;
                        let (_, key, row) = item;
                        async move {
                            let row = update.apply(row).await?;
                            Ok((key, row))
                        }
                    })
                    .try_collect::<Vec<_>>()
                    .await?;

                let column_validation =
                    ColumnValidation::SpecifiedColumns(Rc::from(column_defs), columns_to_update);
                validate_unique(
                    &storage,
                    &table_name,
                    column_validation,
                    rows.iter().map(|r| &r.1),
                )
                .await?;

                Ok(rows)
            });
            let num_rows = rows.len();
            storage
                .update_data(rows)
                .await
                .map(|(storage, _)| (storage, Payload::Update(num_rows)))
        }
        Statement::Delete {
            table_name,
            selection,
        } => {
            let keys = try_block!(storage, {
                let table_name = get_name(&table_name)?;
                let columns = Rc::from(fetch_columns(&storage, table_name).await?);
                let filter = Filter::new(&storage, selection.as_ref(), None, None);

                fetch(&storage, table_name, columns, filter)
                    .await?
                    .map_ok(|(_, key, _)| key)
                    .try_collect::<Vec<_>>()
                    .await
            });

            let num_keys = keys.len();

            storage
                .delete_data(keys)
                .await
                .map(|(storage, _)| (storage, Payload::Delete(num_keys)))
        }*/
        //- Selection
        Statement::Query(query_value) => {
            let result = try_into!(storage, query(&storage, *query_value.clone()).await);
            let (labels, rows) = result;
            let rows = rows.into_iter().map(Row).collect(); // I don't like this. TODO
            let payload = Payload::Select { labels, rows };
            Ok((storage, payload))
        }
        _ => Err((storage, ExecuteError::QueryNotSupported.into())),
    }
}
