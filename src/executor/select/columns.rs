use {
    super::{super::recipe::manual::ObjectName, SelectError},
    crate::{executor::fetch::fetch_columns, store::Store, Result},
    rayon::prelude::*,
    std::fmt::Debug,
};

pub async fn get_columns<'a, T: 'static + Debug>(
    storage: &'a dyn Store<T>,
    table_name: &str,
) -> Result<Vec<ObjectName>> {
    Ok(fetch_columns(storage, table_name)
        .await?
        .into_iter()
        .map(|column| vec![table_name.into(), column.value])
        .collect::<Vec<Vec<String>>>())
}

pub fn needed_column_indexes(
    needed_columns: Vec<ObjectName>,
    all_columns: Vec<ObjectName>,
) -> Result<Vec<usize>> {
    // TODO: Handle amiguous names
    needed_columns
        .into_par_iter()
        .map(|needed_column| {
            all_columns
                .clone()
                .into_iter()
                .enumerate()
                .find(|(_index, column)| {
                    !needed_column
                        .iter()
                        .rev()
                        .zip(column.iter().rev())
                        .any(|(needed_column_part, column_part)| needed_column_part != column_part)
                }) // TODO: Improve
                .map(|(index, _)| index)
                .ok_or(SelectError::ColumnNotFound(needed_column).into())
        })
        .collect()
}
