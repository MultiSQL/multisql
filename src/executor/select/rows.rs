use {
    super::{super::recipe::Recipe, SelectError},
    crate::{store::Store, Result, Row, Value},
    rayon::prelude::*,
    std::fmt::Debug,
};

pub async fn get_rows<'a, T: 'static + Debug>(
    storage: &'a dyn Store<T>,
    table_name: &str,
) -> Result<Vec<Row>> {
    storage
        .scan_data(table_name)
        .await?
        .map(|result| result.map(|(_, row)| row))
        .collect::<Result<Vec<Row>>>()
}

pub fn confirm_rows(
    joined_rows: Vec<Row>,
    needed_column_indexes: Vec<usize>,
    recipe: Recipe,
) -> Result<Vec<Row>> {
    let check_rows = condensed_column_rows(needed_column_indexes, joined_rows.clone())?;

    joined_rows
        .into_iter() // Want to parallelise but cannot due to error not having send. TODO.
        .zip(check_rows)
        .filter_map(
            |(join_row, check_row)| match recipe.clone().confirm(&check_row) {
                Ok(true) => Some(Ok(join_row)),
                Ok(false) => None,
                Err(error) => Some(Err(error)),
            },
        )
        .collect::<Result<Vec<Row>>>()
}

pub fn condensed_column_rows(
    needed_column_indexes: Vec<usize>,
    rows: Vec<Row>,
) -> Result<Vec<Row>> {
    rows.into_par_iter()
        .map(|row| {
            needed_column_indexes
                .clone()
                .into_iter()
                .map(|index| {
                    row.get_value(index)
                        .map(|row| row.clone()) // This is a very heavy use function and so this is very expensive. TODO: Improve.
                        .ok_or(SelectError::ReportableLostColumn.into())
                })
                .collect::<Result<Vec<Value>>>()
                .map(Row)
        })
        .collect()
}
