use std::collections::HashMap;

use crate::IndexFilter;

use {
	super::{
		Ingredient, MetaRecipe, Method, Recipe, RecipeError, RecipeUtilities, Resolve, SimplifyBy,
	},
	crate::{
		executor::types::{ColumnInfo, Row},
		Result, Value,
	},
	fstrings::*,
};

#[derive(Debug, Clone, Default)]
pub struct PlannedRecipe {
	pub recipe: Recipe,
	pub needed_column_indexes: Vec<Option<usize>>,
	pub aggregates: Vec<Recipe>,
}

impl PlannedRecipe {
	pub const TRUE: Self = Self {
		recipe: Recipe::TRUE,
		needed_column_indexes: vec![],
		aggregates: vec![],
	};
	pub fn new(meta_recipe: MetaRecipe, columns: &Vec<ColumnInfo>) -> Result<Self> {
		let MetaRecipe { recipe, meta } = meta_recipe;
		let aggregates = meta.aggregates;
		let needed_column_indexes = meta
			.objects
			.into_iter()
			.map(|needed_column| {
				needed_column
					.map(|needed_column| {
						let needed_column_index_options: Vec<usize> = columns
							.iter()
							.enumerate()
							.filter_map(|(index, column)| {
								if column == &needed_column {
									Some(index.clone())
								} else {
									None
								}
							})
							.collect();
						match needed_column_index_options.len() {
							0 => Err(RecipeError::MissingColumn(needed_column).into()),
							1 => Ok(Some(needed_column_index_options[0])),
							_ => Err(RecipeError::AmbiguousColumn(needed_column).into()),
						}
					})
					.unwrap_or(Ok(None))
			})
			.collect::<Result<Vec<Option<usize>>>>()?;

		Ok(Self {
			recipe,
			needed_column_indexes,
			aggregates,
		})
	}
	pub fn new_constraint(
		meta_recipe: MetaRecipe,
		columns: &Vec<ColumnInfo>,
	) -> Result<(Self, HashMap<String, IndexFilter>)> {
		let mut new = Self::new(meta_recipe, columns)?;
		let indexed_table_columns = columns.clone().into_iter().enumerate().fold(
			HashMap::new(),
			|mut tables: HashMap<String, Vec<(usize, String)>>, (index, column)| {
				if let Some(index_name) = new
					.needed_column_indexes
					.iter()
					.find(|need_index| need_index == &&Some(index))
					.and_then(|_| column.index.clone())
				{
					let col_table = column.table.name;
					if let Some(table) = tables.get_mut(&col_table) {
						table.push((index, index_name));
					} else {
						tables.insert(col_table, vec![(index, index_name)]);
					}
				}
				tables
			},
		);

		let indexed_column_tables = indexed_table_columns.into_iter().fold(
			HashMap::new(),
			|mut indexed_columns, (table, columns)| {
				columns.into_iter().for_each(|(column, index_name)| {
					indexed_columns.insert(column, (table.clone(), index_name));
				});
				indexed_columns
			},
		);

		let result = new.recipe.reduce_by_index_filter(indexed_column_tables);
		new.recipe = result.0;
		let index_filters = result.1.unwrap_or(HashMap::new());

		Ok((new, index_filters))
	}
	pub fn of_index(index: usize) -> Self {
		Self {
			recipe: Recipe::SINGLE_COLUMN,
			needed_column_indexes: vec![Some(index)],
			aggregates: vec![],
		}
	}
	pub fn confirm_join_constraint(&self, plane_row: &Row, self_row: &Row) -> Result<bool> {
		// Very crucial to have performant, needs *a lot* of optimisation.
		// This is currently not good enough.
		// For a join such as:
		/*
			SELECT
				*
			FROM
				big_table
				LEFT JOIN bigger_table
					ON big_table.a = LEFT(bigger_table.b, 3)
				LEFT JOIN biggest_table
					ON big_table.c = (biggest_table.d + 1)
		*/
		/*
			Where:
				(a) big_table     rows =   1 000,
				(b) bigger_table  rows =  10 000,
				(c) biggest_table rows = 100 000,
		*/
		// This will run a * b * c times (1 000 000 000 000 (1e+12)(one trillion) times).
		// This isn't a particularly unusual query for a big database to run.
		// Note that the number of times this runs can, will and should be optimised by reducing the number of rows that need to be compared with good planning scenarios.
		// All of the above (obviously) applies to all functions used in this function.
		let mut plane_row = plane_row.clone();
		plane_row.extend(self_row.clone());

		self.confirm_constraint(&plane_row)
	}
	pub fn confirm_constraint(&self, row: &Row) -> Result<bool> {
		let solution = self
			.clone()
			.simplify_by_row_simple(row)?
			.confirm_or_err(RecipeError::MissingComponents.into())?;
		Ok(matches!(solution, Value::Bool(true)))
	}
	fn simplify_by_row_simple(self, row: &Row) -> Result<Recipe> {
		let row = self.condense_row(row)?;
		self.recipe.simplify(SimplifyBy::Row(&row))
	}
	fn condense_row(&self, row: &Row) -> Result<Row> {
		self.needed_column_indexes
			.iter()
			.map(|index| {
				index
					.map(|index| {
						Ok(row
							.get(index)
							.ok_or_else(|| {
								RecipeError::MissingColumn(vec![
									String::from("Unreachable"),
									f!("{row_len=:?} {index=:?}", row_len = row.len()),
								])
							})?
							.clone())
					})
					.unwrap_or(Ok(Value::Null))
			})
			.collect::<Result<Vec<Value>>>()
	}
	pub fn simplify_by_row(self, row: &Row) -> Result<Self> {
		let row = self.condense_row(row)?;
		let recipe = self.recipe.simplify(SimplifyBy::Row(&row))?;
		let aggregates = self
			.aggregates
			.into_iter()
			.map(|aggregate| aggregate.simplify(SimplifyBy::Row(&row)))
			.collect::<Result<Vec<Recipe>>>()?;
		let needed_column_indexes = self.needed_column_indexes;
		Ok(Self {
			recipe,
			aggregates,
			needed_column_indexes,
		})
	}
	pub fn accumulate(&mut self, other: Self) -> Result<()> {
		self.aggregates = self
			.aggregates
			.clone() // TODO: Don't clone
			.into_iter()
			.zip(other.aggregates)
			.map(|(self_agg, other_agg)| {
				let (operator, self_val) = if let Recipe::Method(self_agg) = self_agg {
					if let Method::Aggregate(operator, recipe) = *self_agg {
						let value = recipe
							.confirm_or_err(RecipeError::UnreachableAggregatationFailed.into())?;
						(operator, value)
					} else {
						return Err(RecipeError::UnreachableNotAggregate(format!(
							"{:?}",
							self_agg
						))
						.into());
					}
				} else {
					return Err(RecipeError::UnreachableNotMethod(format!("{:?}", self_agg)).into());
				};

				let other_val = if let Recipe::Method(other_agg) = other_agg {
					if let Method::Aggregate(_, recipe) = *other_agg {
						let value = recipe
							.confirm_or_err(RecipeError::UnreachableAggregatationFailed.into())?;
						value
					} else {
						return Err(RecipeError::UnreachableNotAggregate(format!(
							"{:?}",
							other_agg
						))
						.into());
					}
				} else {
					return Err(
						RecipeError::UnreachableNotMethod(format!("{:?}", other_agg)).into(),
					);
				};
				let value = Recipe::Ingredient(Ingredient::Value(operator(self_val, other_val)?));
				Ok(Recipe::Method(Box::new(Method::Aggregate(operator, value))))
			})
			.collect::<Result<Vec<Recipe>>>()?;
		Ok(())
	}
	pub fn finalise_accumulation(self) -> Result<Value> {
		let accumulated = self
			.aggregates
			.into_iter()
			.map(|agg| {
				if let Recipe::Method(method) = agg {
					if let Method::Aggregate(_, Recipe::Ingredient(Ingredient::Value(value))) =
						*method
					{
						return Ok(if let Value::Internal(value) = value {
							Value::I64(value)
						} else {
							value
						});
					}
				}
				Err(RecipeError::UnreachableAggregateFailed.into())
			})
			.collect::<Result<_>>()?;
		self.recipe
			.simplify(SimplifyBy::CompletedAggregate(accumulated))?
			.confirm_or_err(RecipeError::UnreachableAggregateFailed.into())
	}
	pub fn get_label(
		&self,
		selection_index: usize,
		include_table: bool,
		columns: &Vec<ColumnInfo>,
	) -> String {
		if let Recipe::Ingredient(Ingredient::Column(_)) = self.recipe {
			self.needed_column_indexes
				.get(0)
				.map(|index| index.map(|index| columns.get(index)))
				.flatten()
				.flatten()
				.map(|column| {
					if include_table {
						format!("{}.{}", column.table.name, column.name)
					} else {
						column.name.clone()
					}
				})
		} else {
			None
		}
		.unwrap_or(format!("unnamed_{}", selection_index))
	}
}
