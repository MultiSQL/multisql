use {
    super::{auto_increment, columns_to_positions, validate, validate_unique},
    crate::{
        data::{get_name, Schema},
        executor::types::{ComplexColumnName, Row as VecRow},
        macros::{try_block, try_into},
        result::MutResult,
        store::{AlterTable, AutoIncrement, Store, StoreMut},
        ExecuteError, MetaRecipe, Payload, PlannedRecipe, RecipeUtilities, Result, Row,
    },
    sqlparser::ast::{Assignment, ColumnDef, Expr, ObjectName},
    std::fmt::Debug,
};

pub async fn update<
    Key: 'static + Debug,
    Storage: Store<Key> + StoreMut<Key> + AlterTable + AutoIncrement,
>(
    storage: Storage,
    table_name: &ObjectName,
    selection: &Option<Expr>,
    assignments: &Vec<Assignment>,
) -> MutResult<Storage, Payload> {
    let (keys, rows, table_name, column_defs): (Vec<Key>, Vec<VecRow>, &str, Vec<ColumnDef>) =
        try_block!(storage, {
            let table_name = get_name(table_name)?;
            let Schema { column_defs, .. } = storage
                .fetch_schema(table_name)
                .await?
                .ok_or(ExecuteError::TableNotExists)?;

            let columns = column_defs
                .clone()
                .into_iter()
                .map(|column_def| {
                    let ColumnDef { name, .. } = column_def;
                    ComplexColumnName {
                        name: name.value,
                        table: (None, String::new()),
                    }
                })
                .collect();

            let filter = selection
                .clone()
                .map(|selection| PlannedRecipe::new(MetaRecipe::new(selection)?, &columns))
                .unwrap_or(Ok(PlannedRecipe::TRUE))?;

            let assignments = assignments
                .into_iter()
                .map(|assignment| {
                    let Assignment { id, value } = assignment;
                    let column_name = id.value.clone();
                    let column_compare = vec![column_name.clone()];
                    let index = columns
                        .iter()
                        .position(|column| column == &column_compare)
                        .ok_or(ExecuteError::ColumnNotFound)?;
                    let recipe = PlannedRecipe::new(MetaRecipe::new(value.clone())?, &columns)?;
                    Ok((index, recipe))
                })
                .collect::<Result<Vec<(usize, PlannedRecipe)>>>()?;

            let keyed_rows = storage
                .scan_data(table_name)
                .await?
                .into_iter()
                .filter_map(|row_result| {
                    let (key, row) = match row_result {
                        Ok(keyed_row) => keyed_row,
                        Err(error) => return Some(Err(error)),
                    };

                    let row = row.0;

                    let confirm_constraint = filter.confirm_constraint(&row.clone());
                    if let Ok(false) = confirm_constraint {
                        return None;
                    } else if let Err(error) = confirm_constraint {
                        return Some(Err(error));
                    }
                    let row = row
                        .iter()
                        .enumerate()
                        .map(|(index, old_value)| {
                            assignments
                                .iter()
                                .find(|(assignment_index, _)| assignment_index == &index)
                                .map(|(_, assignment_recipe)| {
                                    assignment_recipe.clone().simplify_by_row(&row)?.confirm()
                                })
                                .unwrap_or(Ok(old_value.clone()))
                        })
                        .collect::<Result<VecRow>>();
                    Some(row.map(|row| (key, row)))
                })
                .collect::<Result<Vec<(Key, VecRow)>>>()?;

            let column_positions = columns_to_positions(&column_defs, &[])?;
            let (keys, rows): (Vec<Key>, Vec<VecRow>) = keyed_rows.into_iter().unzip();
            let rows = validate(&column_defs, &column_positions, rows)?;

            Ok((keys, rows, table_name.as_str(), column_defs))
        });
    #[cfg(feature = "auto-increment")]
    let (storage, rows) = auto_increment(storage, table_name, &column_defs, rows).await?;
    try_into!(
        storage,
        validate_unique(&storage, table_name, &column_defs, &rows).await
    );
    let keyed_rows: Vec<(Key, Row)> = keys.into_iter().zip(rows.into_iter().map(Row)).collect();
    let num_rows = keyed_rows.len();
    storage
        .update_data(keyed_rows)
        .await
        .map(|(storage, _)| (storage, Payload::Update(num_rows)))
}