use {
	super::{Ingredient, Method, Recipe, RecipeError, RecipeUtilities},
	crate::{types::Row, Result, Value},
};

#[derive(Clone)]
pub enum SimplifyBy<'a> {
	Basic,
	OptRow(&'a Vec<Option<Value>>),
	Row(&'a Row),
	CompletedAggregate(Vec<Value>),
}

pub trait Resolve
where
	Self: Sized,
{
	fn simplify(self, component: SimplifyBy) -> Result<Self>;
}

impl Resolve for Recipe {
	fn simplify(self, component: SimplifyBy) -> Result<Self> {
		match self {
			Recipe::Ingredient(ingredient) => {
				ingredient.simplify(component).map(Recipe::Ingredient)
			}
			Recipe::Method(method) => method.simplify(component).map(|method| {
				if let Method::Value(value) = method {
					Recipe::Ingredient(Ingredient::Value(value))
				} else {
					Recipe::Method(Box::new(method))
				}
			}),
		}
	}
}

impl Resolve for Ingredient {
	fn simplify(self, component: SimplifyBy) -> Result<Self> {
		Ok(match self {
			Ingredient::Column(index) => {
				if let SimplifyBy::Row(row) = component {
					Ingredient::Value(row.get(index).ok_or(RecipeError::UnreachableNoRow)?.clone())
				} else if let SimplifyBy::OptRow(row) = component {
					row.get(index)
						.and_then(Clone::clone)
						.map(Ingredient::Value)
						.unwrap_or(self)
				} else {
					self
				}
			}
			Ingredient::Aggregate(index) => {
				if let SimplifyBy::CompletedAggregate(values) = component {
					Ingredient::Value(values.get(index).ok_or(RecipeError::Unreachable)?.clone())
				} else {
					self
				}
			}
			Ingredient::Value(..) => self, // Already simple!
		})
	}
}

#[allow(clippy::if_same_then_else)] // No idea what Clippy is trying to say here
#[allow(clippy::collapsible_else_if)] // Intentional for clarity
impl Resolve for Method {
	fn simplify(self, component: SimplifyBy) -> Result<Self> {
		Ok(match self {
			Method::UnaryOperation(operator, recipe) => {
				let recipe = recipe.simplify(component)?;
				if let Some(value) = recipe.as_solution() {
					Method::Value(operator(value)?)
				} else {
					Method::UnaryOperation(operator, recipe)
				}
			}
			Method::BinaryOperation(operator, left, right) => {
				let left = left.simplify(component.clone())?;

				if let Some(Value::Bool(value)) = left.as_solution() {
					// Optimisation -- is this a good idea?
					if !value {
						// Clippy didn't like this without "as usize"
						if operator as usize == Value::and as usize {
							return Ok(Method::Value(Value::Bool(false)));
						}
					} else {
						if operator as usize == Value::or as usize {
							return Ok(Method::Value(Value::Bool(true)));
						}
					}
				}

				let right = right.simplify(component)?;

				if let (Some(left), Some(right)) = (left.as_solution(), right.as_solution()) {
					Method::Value(operator(left, right)?)
				} else {
					Method::BinaryOperation(operator, left, right)
				}
			}
			Method::Function(function, arguments) => {
				let arguments = arguments
					.into_iter()
					.map(|argument| argument.simplify(component.clone()))
					.collect::<Result<Vec<Recipe>>>()?;
				if let Some(arguments) = arguments
					.iter()
					.map(|argument| argument.as_solution())
					.collect::<Option<Vec<Value>>>()
				{
					Method::Value(function(arguments)?)
				} else {
					Method::Function(function, arguments)
				}
			}
			Method::Cast(data_type, recipe) => {
				let recipe = recipe.simplify(component)?;
				if let Some(value) = recipe.as_solution() {
					Method::Value(value.cast_datatype(&data_type)?)
				} else {
					Method::Cast(data_type, recipe)
				}
			}

			Method::Case {
				operand,
				cases,
				else_result,
			} => {
				let operand = operand
					.map(|operand| operand.simplify(component.clone()))
					.transpose()?;
				let else_result = else_result
					.map(|else_result| else_result.simplify(component.clone()))
					.transpose()?;
				let cases = cases
					.into_iter()
					.map(|(condition, result)| {
						Ok((
							condition.simplify(component.clone())?,
							result.simplify(component.clone())?,
						))
					})
					.collect::<Result<Vec<(Recipe, Recipe)>>>()?;

				if let Some(None) = operand.clone().map(|operand| operand.as_solution()) {
					Method::Case {
						operand,
						cases,
						else_result,
					}
				} else if let Some(None) = else_result
					.clone()
					.map(|else_result| else_result.as_solution())
				{
					Method::Case {
						operand,
						cases,
						else_result,
					}
				} else if let Some(cases) = cases
					.iter()
					.map(|(condition, result)| {
						Some((condition.as_solution()?, result.as_solution()?))
					})
					.collect::<Option<Vec<(Value, Value)>>>()
				{
					let operand = operand.map(|operand| operand.as_solution());
					let else_result = else_result
						.map(|else_result| else_result.as_solution())
						.unwrap_or(Some(Value::NULL))
						.unwrap();
					if let Some(operand) = operand {
						let operand = operand.unwrap();
						Method::Value(
							cases
								.into_iter()
								.find_map(|(condition, result)| {
									if operand == condition {
										Some(result)
									} else {
										None
									}
								})
								.unwrap_or(else_result),
						)
					} else {
						Method::Value(
							cases
								.into_iter()
								.find_map(|(condition, result)| {
									if matches!(condition, Value::Bool(true)) {
										Some(result)
									} else {
										None
									}
								})
								.unwrap_or(else_result),
						)
					}
				} else {
					Method::Case {
						operand,
						cases,
						else_result,
					}
				}
			}

			Method::Value(..) => return Err(RecipeError::Unreachable.into()),

			// This will only occur for a special aggregate simplify
			Method::Aggregate(operator, recipe) => {
				Method::Aggregate(operator, recipe.simplify(component)?)
			}
		})
	}
}
