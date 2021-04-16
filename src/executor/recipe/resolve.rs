use {
    super::{Ingredient, Method, Recipe, RecipeError, RecipeUtilities},
    crate::{executor::types::Row, Result, Value},
};

#[derive(Clone)]
pub enum SimplifyBy<'a> {
    Basic,
    Row(&'a Row),
    CompletedAggregate(Vec<Value>),
}

pub trait Resolve {
    fn simplify(self, component: SimplifyBy) -> Result<Self>
    where
        Self: Sized;
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
                    Ingredient::Value(row.get(index).ok_or(RecipeError::Unreachable)?.clone())
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
                let (left, right) = (
                    left.simplify(component.clone())?,
                    right.simplify(component)?,
                );
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

            Method::Value(..) => return Err(RecipeError::Unreachable.into()),
            Method::Aggregate(..) => return Err(RecipeError::Unreachable.into()),
        })
    }
}
