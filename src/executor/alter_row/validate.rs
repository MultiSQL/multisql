use {
	crate::{
		executor::types::Row, Ingredient, Recipe, RecipeUtilities,
		Resolve, Result, SimplifyBy, Value, ValueType, Column, ValueDefault
	},
	rayon::prelude::*,
	serde::Serialize,
	sqlparser::ast::Ident,
	thiserror::Error as ThisError,
};

#[derive(ThisError, Serialize, Debug, PartialEq)]
pub enum ValidateError {
	#[error("expected value for column which neither accepts NULL nor has a default")]
	MissingValue,
	#[error("wrong number of values in insert statement")]
	WrongNumberOfValues,
	#[error("default value failed to be calculated")]
	BadDefault,
	#[error("column '{0}' not found")]
	ColumnNotFound(String),
	#[error("found duplicate value on unique field")]
	//#[error("column '{0}' is unique but '{1:?}' was attempted to be stored twice")]
	DuplicateEntryOnUniqueField, /*(String, Value)*/

	#[error("this should be impossible, please report")]
	UnreachableUniqueValues,
}

pub fn columns_to_positions(column_defs: &[Column], columns: &[Ident]) -> Result<Vec<usize>> {
	if columns.is_empty() {
		Ok((0..column_defs.len()).collect())
	} else {
		columns
			.iter()
			.map(|stated_column| {
				column_defs
					.iter()
					.position(|column_def| stated_column.value == column_def.name)
					.ok_or_else(|| {
						ValidateError::ColumnNotFound(stated_column.value.clone()).into()
					})
			})
			.collect::<Result<Vec<usize>>>()
	}
}

pub fn validate(
	columns: &[Column],
	stated_columns: &[usize],
	rows: &mut Vec<Row>,
) -> Result<()> {
	if rows.iter().any(|row| row.len() != stated_columns.len()) {
		return Err(ValidateError::WrongNumberOfValues.into());
	}

	let column_info = columns
		.iter()
		.enumerate()
		.map(|(column_def_index, column)| {
			let index = stated_columns
				.iter()
				.position(|stated_column| stated_column == &column_def_index);

			let nullable = column.is_nullable || column.default.is_some();

			let failure_recipe = if let Some(ValueDefault::Recipe(expr)) = &column.default {
				Some(Recipe::new_without_meta(expr.clone())?)
			} else if nullable {
				Some(Recipe::NULL)
			} else {
				None
			};
			Ok((index, failure_recipe, nullable, &column.data_type))
		})
		.collect::<Result<Vec<(Option<usize>, Option<Recipe>, bool, &ValueType)>>>()?;
	*rows = rows.into_par_iter()
		.map(|row| {
			column_info
				.iter()
				.map(|(index, failure_recipe, nullable, data_type)| {
					index
						.map(|index| {
							row.get(index).map(|value| {
								let mut value = value.clone();
								if let Err(error) = value.validate_null(*nullable) {
									value = if let Some(fallback) = failure_recipe.clone() {
										if !matches!(
											fallback,
											Recipe::Ingredient(Ingredient::Value(Value::Null))
										) {
											fallback
												.simplify(SimplifyBy::Basic)?
												.as_solution()
												.ok_or(ValidateError::BadDefault)?
										} else {
											return Err(error);
										}
									} else {
										return Err(error);
									}
								}
								value.is(data_type)?;
								Ok(value)
							})
						})
						.flatten()
						.unwrap_or({
							if let Some(recipe) = failure_recipe.clone() {
								recipe
									.simplify(SimplifyBy::Basic)?
									.as_solution()
									.ok_or_else(|| ValidateError::BadDefault.into())
							} else {
								Err(ValidateError::MissingValue.into())
							}
						})
				})
				.collect::<Result<Row>>()
		})
		.collect::<Result<Vec<Row>>>()?;
	Ok(())
}
