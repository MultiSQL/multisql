use {
    super::recipe::{Join, Manual, Recipe},
    crate::{executor::fetch::fetch_columns, store::Store, Result, Row, Value},
    futures::stream::{self, StreamExt, TryStreamExt},
    serde::Serialize,
    sqlparser::ast::{Ident, Query},
    std::fmt::Debug,
    thiserror::Error as ThisError,
};

#[derive(ThisError, Serialize, Debug, PartialEq)]
pub enum SelectError {
    #[error("unimplemented! select on two or more than tables are not supported")]
    TooManyTables,

    #[error("table alias not found: {0}")]
    TableAliasNotFound(String),

    #[error("unreachable!")]
    Unreachable,
}

type ColumnsAndRows = (Vec<Vec<Ident>>, Vec<Row>);

pub async fn select<'a, T: 'static + Debug>(
    storage: &'a dyn Store<T>,
    query: &'a Query,
) -> Result<(Vec<String> /* Labels */, Vec<Row>)> {
    let manual = Manual::write(query.clone())?;
    let Manual {
        initial_table,
        joins,
        selections,
        columns: needed_columns,
        groups: _,
        constraint: _,
        contains_aggregate: _,
    } = manual;

    let table_name = initial_table.1.as_str();
    let columns = get_columns(storage, table_name).await?;
    let rows = get_rows(storage, table_name).await?;

    let joins: Vec<Result<Join>> = joins.into_iter().map(Ok).collect();

    let (columns, rows) = stream::iter(joins)
        .try_fold((columns, rows), |columns_and_rows, join| {
            join_table(storage, columns_and_rows, join)
        })
        .await?;

    // TODO: Constraint

    let needed_column_indexes = needed_column_indexes(needed_columns, columns.clone());
    let rows = condensed_column_rows(needed_column_indexes, rows);

    let cooked_rows = rows
        .into_iter()
        .map(|row| {
            row.0
                .iter()
                .enumerate()
                .map(|(index, _column)| {
                    selections
                        .get(index)
                        .unwrap() /* TODO: Handle */
                        .recipe
                        .clone()
                        .must_solve(&row)
                })
                .collect::<Result<Vec<Value>>>()
                .map(Row)
        })
        .collect::<Result<Vec<Row>>>()?;

    let labels = selections
        .into_iter()
        .enumerate()
        .map(|(index, selection)| {
            selection
                .alias
                .unwrap_or(
                    columns
                        .get(index)
                        .unwrap() /* TODO: Handle */
                        .get(0)
                        .unwrap() /* TODO: Handle */
                        .clone(),
                )
                .value
        })
        .collect();

    // TODO: Group

    Ok((labels, cooked_rows))
}

async fn join_table<'a, T: 'static + Debug>(
    storage: &'a dyn Store<T>,
    columns_and_rows: ColumnsAndRows,
    join: Join,
) -> Result<ColumnsAndRows> {
    let (columns, rows) = columns_and_rows;
    let mut columns = columns;
    let (table, (_join_operation, recipe, needed_columns)) = join;
    let table_name = table.1.as_str();

    let mut join_columns = get_columns(storage, table_name).await?;
    columns.append(&mut join_columns);

    let join_rows = get_rows(storage, table_name).await?;

    let joined_rows = rows.into_iter().fold(vec![], |joined_rows, to_join| {
        join_fold(join_rows.clone(), joined_rows, to_join)
    });

    let needed_column_indexes: Vec<usize> = needed_column_indexes(needed_columns, columns.clone());

    // Only inner for now
    let confirmed_joined_rows = confirm_rows(joined_rows, needed_column_indexes, recipe);

    Ok((columns, confirmed_joined_rows))
}

fn confirm_rows(
    joined_rows: Vec<Row>,
    needed_column_indexes: Vec<usize>,
    recipe: Recipe,
) -> Vec<Row> {
    let check_rows = condensed_column_rows(needed_column_indexes, joined_rows.clone());
    joined_rows
        .into_iter()
        .enumerate()
        .filter(
            |(index, _row)| recipe.clone().confirm(&check_rows[*index]).unwrap(), /* TODO: Handle */
        )
        .map(|(_, row)| row)
        .collect()
}

async fn get_columns<'a, T: 'static + Debug>(
    storage: &'a dyn Store<T>,
    table_name: &str,
) -> Result<Vec<Vec<Ident>>> {
    Ok(fetch_columns(storage, table_name)
        .await?
        .into_iter()
        .map(|column| vec![column])
        .collect())
}

async fn get_rows<'a, T: 'static + Debug>(
    storage: &'a dyn Store<T>,
    table_name: &str,
) -> Result<Vec<Row>> {
    storage
        .scan_data(table_name)
        .await?
        .map(|result| result.map(|(_, row)| row))
        .collect::<Result<Vec<Row>>>()
}

fn needed_column_indexes(
    needed_columns: Vec<Vec<Ident>>,
    all_columns: Vec<Vec<Ident>>,
) -> Vec<usize> {
    needed_columns
        .into_iter()
        .map(|needed_column| {
            all_columns
                .clone()
                .into_iter()
                .enumerate()
                .find(|(_index, column)| needed_column == *column)
                .map(|(index, _)| index)
                .unwrap() /* TODO: Handle */
        })
        .collect()
}

fn condensed_column_rows(needed_column_indexes: Vec<usize>, rows: Vec<Row>) -> Vec<Row> {
    rows.into_iter()
        .map(|row| {
            Row(needed_column_indexes
                .clone()
                .into_iter()
                .map(
                    |index| row.get_value(index).unwrap().clone(), /* TODO: Handle */
                )
                .collect())
        })
        .collect()
}

fn join_fold(join_rows: Vec<Row>, mut joined_rows: Vec<Row>, to_join: Row) -> Vec<Row> {
    let mut join_rows = join_rows
        .into_iter()
        .map(|mut join_row| {
            join_row.0.append(&mut to_join.0.clone());
            join_row
        })
        .collect::<Vec<Row>>();
    joined_rows.append(&mut join_rows);
    joined_rows
}
