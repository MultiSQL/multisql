use {
    crate::{
        data::schema::ColumnDefExt, executor::types::Row, InsertError, Recipe, RecipeUtilities,
        Resolve, Result, SimplifyBy,
    },
    sqlparser::ast::{ColumnDef, DataType, Ident},
};

pub fn validate(
    column_defs: &[ColumnDef],
    stated_columns: &[Ident],
    rows: Vec<Row>,
) -> Result<Vec<Row>> {
    // FAIL: No mut
    let table_columns_count = column_defs.len();
    let selection_columns_count = if stated_columns.is_empty() {
        table_columns_count
    } else {
        stated_columns.len()
    };

    if rows.iter().any(|row| row.len() != selection_columns_count) {
        return Err(InsertError::WrongNumberOfValues.into());
    }

    let column_info = column_defs
        .into_iter()
        .enumerate()
        .map(|(column_def_index, column_def)| {
            let ColumnDef {
                name, data_type, ..
            } = column_def;

            let column_name = name.to_string();
            let index = if stated_columns.is_empty() {
                Some(column_def_index)
            } else {
                stated_columns
                    .iter()
                    .position(|stated_column| stated_column.value == column_name)
            };

            let nullable = column_def.is_nullable();
            #[cfg(feature = "auto-increment")]
            let nullable = nullable || column_def.is_auto_incremented();

            let failure_recipe = if let Some(default) = column_def.get_default() {
                Some(Recipe::new_without_meta(default.clone())?)
            } else if nullable {
                Some(Recipe::NULL)
            } else {
                None
            };
            Ok((index, failure_recipe, nullable, data_type))
        })
        .collect::<Result<Vec<(Option<usize>, Option<Recipe>, bool, &DataType)>>>()?;
    rows.into_iter()
        .map(|row| {
            column_info
                .iter()
                .map(|(index, failure_recipe, nullable, data_type)| {
                    index
                        .map(|index| {
                            row.get(index).map(|value| {
                                let mut value = value.clone();
                                value.validate_null(nullable.clone())?;
                                value = value.validate_type(data_type)?;
                                Ok(value)
                            })
                        })
                        .flatten()
                        .unwrap_or({
                            if let Some(recipe) = failure_recipe.clone() {
                                recipe
                                    .simplify(SimplifyBy::Basic)?
                                    .as_solution()
                                    .ok_or(InsertError::BadDefault.into())
                            } else {
                                Err(InsertError::MissingValue.into())
                            }
                        })
                })
                .collect::<Result<Row>>()
        })
        .collect::<Result<Vec<Row>>>()
}
