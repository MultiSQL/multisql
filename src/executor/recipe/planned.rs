use {
	super::{
		Ingredient, MetaRecipe, Method, Recipe, RecipeError, RecipeUtilities, Resolve, SimplifyBy,
	},
	crate::{
		executor::types::{ComplexColumnName, Row},
		Result, Value,
	},
	fstrings::*,
};

#[derive(Debug, Clone)]
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
	pub fn new(meta_recipe: MetaRecipe, columns: &Vec<ComplexColumnName>) -> Result<Self> {
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
	pub fn aggregate(&self, accumulated: Vec<Value>) -> Result<Vec<Value>> {
		let accumulated = if accumulated.is_empty() {
			vec![Value::Null; self.aggregates.len()]
		} else {
			accumulated
		};
		self.aggregates
			.clone()
			.into_iter()
			.zip(accumulated)
			.map(|(aggregate, accumulated)| {
				if let Recipe::Method(aggregate) = aggregate {
					if let Method::Aggregate(operator, recipe) = *aggregate {
						let value = recipe
							.confirm_or_err(RecipeError::UnreachableAggregatationFailed.into())?;
						operator(value, accumulated)
					} else {
						Err(RecipeError::UnreachableNotAggregate(format!("{:?}", aggregate)).into())
					}
				} else {
					Err(RecipeError::UnreachableNotMethod(format!("{:?}", aggregate)).into())
				}
			})
			.collect::<Result<Vec<Value>>>()
	}
	pub fn accumulate(&mut self, other: Self) -> Result<()> {
		self.aggregates = self.aggregates
			.into_iter()
			.zip(other.aggregates)
			.map(|(self_agg, other_agg)| {
				let (operator, self_val) = if let Recipe::Method(self_agg) = self_agg {
					if let Method::Aggregate(operator, recipe) = *self_agg {
						let value = recipe
							.confirm_or_err(RecipeError::UnreachableAggregatationFailed.into())?;
						(operator, value)
					} else {
						return Err(RecipeError::UnreachableNotAggregate(format!("{:?}", self_agg)).into())
					}
				} else {
					return Err(RecipeError::UnreachableNotMethod(format!("{:?}", self_agg)).into())
				};

				let other_val = if let Recipe::Method(other_agg) = other_agg {
					if let Method::Aggregate(_, recipe) = *other_agg {
						let value = recipe
							.confirm_or_err(RecipeError::UnreachableAggregatationFailed.into())?;
						value
					} else {
						return Err(RecipeError::UnreachableNotAggregate(format!("{:?}", other_agg)).into())
					}
				} else {
					return Err(RecipeError::UnreachableNotMethod(format!("{:?}", other_agg)).into())
				};
				let value = Recipe::Ingredient(Ingredient::Value(operator(self_val, other_val)?));
				Ok(Recipe::Method(Box::new(Method::Aggregate(operator, value))))
			})
			.collect::<Result<Vec<Recipe>>>()?;
			Ok(())
	}
	pub fn solve_by_aggregate(self, accumulated: Vec<Value>) -> Result<Value> {
		self.recipe
			.simplify(SimplifyBy::CompletedAggregate(accumulated))?
			.confirm_or_err(RecipeError::UnreachableAggregateFailed.into())
	}
	pub fn get_label(
		&self,
		selection_index: usize,
		include_table: bool,
		columns: &Vec<ComplexColumnName>,
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
