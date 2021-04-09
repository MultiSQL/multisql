use {
    super::{
        super::recipe::{manual::ColumnsAndRows, Join},
        confirm_rows, get_columns, get_rows, needed_column_indexes,
    },
    crate::{store::Store, Result, Row},
    rayon::prelude::*,
    std::fmt::Debug,
};

pub async fn join_table<'a, T: 'static + Debug>(
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

    let needed_column_indexes = needed_column_indexes(needed_columns, columns.clone())?;

    // Only inner for now
    let confirmed_joined_rows = confirm_rows(joined_rows, needed_column_indexes, recipe)?;

    Ok((columns, confirmed_joined_rows))
}

fn join_fold(join_rows: Vec<Row>, mut joined_rows: Vec<Row>, to_join: Row) -> Vec<Row> {
    let mut join_rows = join_rows
        .into_par_iter()
        .map(|mut join_row| {
            join_row.0.append(&mut to_join.0.clone());
            join_row
        })
        .collect::<Vec<Row>>();
    joined_rows.append(&mut join_rows);
    joined_rows
}
