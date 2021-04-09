mod columns;
mod join;
mod rows;

use {
    super::recipe::{
        manual::{column_recipe, LabelledSelection, LabelsAndRows, ObjectName, Selection},
        Ingredient, Join, Keys, Manual, Recipe, Resolve,
    },
    crate::{store::Store, Result, Row, Value},
    columns::{get_columns, needed_column_indexes},
    futures::stream::{self, TryStreamExt},
    join::join_table,
    rows::{condensed_column_rows, confirm_rows, get_rows},
    serde::Serialize,
    sqlparser::ast::Query,
    std::fmt::Debug,
    thiserror::Error as ThisError,
};

#[derive(ThisError, Serialize, Debug, PartialEq)]
pub enum SelectError {
    #[error("unimplemented! select on two or more than tables are not supported")]
    TooManyTables,

    #[error("table alias not found: {0}")]
    TableAliasNotFound(String),

    #[error("column not found: {0:?}")]
    ColumnNotFound(Vec<String>),

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
        groups,
        limit,
        aggregate_selection_indexes,
    } = manual;

    // Subqueries are gross atm. TODO: Make subqueries nicer.
    let selections = selections
        .into_iter()
        .map(|selection| {
            if let Selection::Recipe { recipe, alias } = selection {
                let recipe = recipe.simplify(Some(&Keys { row: None }))?;
                Ok(Selection::Recipe { recipe, alias })
            } else {
                Ok(selection)
            }
        })
        .collect::<Result<Vec<Selection>>>()?;
    let constraint = constraint.simplify(Some(&Keys { row: None }))?;

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
    name.into_iter()
        .last()
        .map(|string| string.clone())
        .unwrap_or(String::new())
}
