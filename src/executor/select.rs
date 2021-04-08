use {
    super::recipe::{
        manual::{
            column_recipe, ColumnsAndRows, LabelledSelection, LabelsAndRows, ObjectName, Selection,
        },
        Ingredient, Join, Manual, Recipe, Resolve,
    },
    crate::{executor::fetch::fetch_columns, store::Store, Result, Row, Value},
    futures::stream::{self, TryStreamExt},
    rayon::prelude::*,
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

    #[error("column could not be found for some reason")]
    ReportableLostColumn,

    #[error("this should be impossible, please report")]
    Unreachable,
}

pub async fn select<'a, T: 'static + Debug>(
    storage: &'a dyn Store<T>,
    query: &'a Query,
) -> Result<LabelsAndRows> {
    let manual = Manual::write(query.clone())?;
    let Manual {
        initial_table,
        joins,
        selections,
        needed_columns,
        constraint,
        groups: _,
        contains_aggregate: _,
        limit,
    } = manual;
    println!("{:?}", constraint); // TODO DEBUG

    let table_name = initial_table.1.as_str();
    let columns = get_columns(storage, table_name).await?;
    let rows = get_rows(storage, table_name).await?;

    let joins: Vec<Result<Join>> = joins.into_iter().map(Ok).collect();

    let (joined_columns, rows) = stream::iter(joins)
        .try_fold((columns, rows), |columns_and_rows, join| {
            join_table(storage, columns_and_rows, join)
        })
        .await?;

    let (selections, needed_columns) =
        expand_selections(joined_columns.clone(), selections, needed_columns);

    let needed_column_indexes = needed_column_indexes(needed_columns.clone(), joined_columns)?;
    let rows = condensed_column_rows(needed_column_indexes, rows)?;

    let cooked_rows = rows
        .into_iter()
        .filter_map(|row| match constraint.clone().confirm(&row) {
            Ok(true) => Some(
                selections
                    .clone()
                    .into_iter()
                    .map(|(recipe, _)| recipe.must_solve(&row))
                    .collect::<Result<Vec<Value>>>()
                    .map(Row),
            ),
            Ok(false) => None,
            Err(error) => Some(Err(error)),
        })
        .collect::<Result<Vec<Row>>>()?;

    let labels = selections.into_iter().map(|(_, label)| label).collect();

    let mut cooked_rows = cooked_rows;
    if let Some(Value::I64(limit)) = limit.simplify(None)?.as_solution() {
        // TODO: Do this less grossly
        cooked_rows.truncate(limit as usize);
    }

    // TODO: Group

    Ok((labels, cooked_rows))
}

fn expand_selections(
    available_columns: Vec<ObjectName>,
    selections: Vec<Selection>,
    mut needed_columns: Vec<ObjectName>,
) -> (Vec<LabelledSelection>, Vec<ObjectName>) {
    let unfolded_labelled_selections = selections.into_iter().map(|selection| match selection {
        Selection::Recipe { recipe, alias } => vec![(
            recipe.clone(),
            alias
                .map(|alias| alias.value)
                .unwrap_or(alias_from_recipe(recipe, &needed_columns)),
        )],
        Selection::Wildcard { qualifier } => available_columns
            .iter()
            .filter(|column| {
                qualifier
                    .clone()
                    .map(|qualifier| {
                        qualifier
                            .iter()
                            .enumerate()
                            .any(|(index, qualifying_part)| {
                                column
                                    .get(index)
                                    .map(|part| part != qualifying_part)
                                    .unwrap_or(true)
                            })
                    })
                    .unwrap_or(true)
            })
            .map(|column| {
                (
                    column_recipe(column.clone(), &mut needed_columns),
                    alias_from_object_name(&column),
                )
            })
            .collect(),
    });
    let labelled_selections =
        unfolded_labelled_selections.fold(vec![], |mut chain, mut labelled_selections| {
            chain.append(&mut labelled_selections);
            chain
        });
    (labelled_selections, needed_columns)
}
fn alias_from_recipe(recipe: Recipe, columns: &Vec<ObjectName>) -> String {
    match recipe {
        Recipe::Ingredient(Ingredient::Column(index)) => columns
            .get(index)
            .map(|column| alias_from_object_name(column)),
        _ => None, // TODO: More
    }
    .unwrap_or(String::new())
}
fn alias_from_object_name(name: &ObjectName) -> String {
    let mut iter = name.into_iter();
    let first_value = iter
        .next()
        .map(|part| part.value.clone())
        .unwrap_or(String::new());
    iter.fold(first_value, |alias, part| {
        format!("{}.{}", alias, part.value.clone())
    })
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

    let needed_column_indexes = needed_column_indexes(needed_columns, columns.clone())?;

    // Only inner for now
    let confirmed_joined_rows = confirm_rows(joined_rows, needed_column_indexes, recipe)?;

    Ok((columns, confirmed_joined_rows))
}

fn confirm_rows(
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
) -> Result<Vec<usize>> {
    needed_columns
        .into_par_iter()
        .map(|needed_column| {
            all_columns
                .clone()
                .into_iter()
                .enumerate()
                .find(|(_index, column)| needed_column == *column)
                .map(|(index, _)| index)
                .ok_or(SelectError::ReportableLostColumn.into())
        })
        .collect()
}

fn condensed_column_rows(needed_column_indexes: Vec<usize>, rows: Vec<Row>) -> Result<Vec<Row>> {
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
