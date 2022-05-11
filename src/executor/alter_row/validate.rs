use {
	crate::{
		executor::types::Row, Column, Error, Recipe, RecipeUtilities, Resolve, Result,
		SimplifyBy, ValueDefault, ValueType,
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

pub fn validate(columns: &[Column], stated_columns: &[usize], rows: &mut Vec<Row>) -> Result<()> {
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
	*rows = rows
		.into_par_iter()
		.map(|row| {
			column_info
				.iter()
				.map(|(index, failure_recipe, nullable, data_type)| {
					let mut value = index
						.and_then(|index| row.get(index).cloned())
						.ok_or(Error::Validate(ValidateError::MissingValue)).and_then(|value| {
						value.validate_null(*nullable).map(|_| value)}).or_else(|_| {
							let recipe = failure_recipe
								.clone()
								.ok_or(Error::Validate(ValidateError::MissingValue))?;
							recipe
								.simplify(SimplifyBy::Basic)?
								.as_solution()
								.ok_or(Error::Validate(ValidateError::BadDefault))
						})?;

					value.is(data_type)?;
					Ok(value)
				})
				.collect::<Result<Row>>()
		})
		.collect::<Result<Vec<Row>>>()?;
	Ok(())
}
