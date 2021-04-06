use {
    super::{Ingredient, Method, Recipe, RecipeError, RecipeKey, RecipeSolution},
    crate::{Result, Value},
};

pub trait Resolve {
    fn solve(self, row: RecipeKey) -> RecipeSolution;
    fn simplify(self, row: RecipeKey) -> Result<Self>
    where
        Self: Sized;
}

impl Resolve for Recipe {
    fn solve(self, row: RecipeKey) -> RecipeSolution {
        match self {
            Recipe::Ingredient(ingredient) => ingredient.solve(row),
            Recipe::Method(method) => method.solve(row),
        }
    }
    fn simplify(self, row: RecipeKey) -> Result<Self> {
        match self {
            Recipe::Ingredient(ingredient) => ingredient.simplify(row).map(Recipe::Ingredient),
            Recipe::Method(method) => method.simplify(row).map(|method| {
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
    fn solve(self, row: RecipeKey) -> RecipeSolution {
        match self {
            Ingredient::Value(value) => Some(Ok(value)),
            Ingredient::Column(index) => row.clone().map(|row| {
                row.get_value(index)
                    .ok_or(
                        RecipeError::Failed(String::from("Couldn't get value with row index!"))
                            .into(),
                    )
                    .map(|value| value.clone())
            }),
        }
    }
    fn simplify(self, row: RecipeKey) -> Result<Self> {
        self.clone()
            .solve(row)
            .map(|result| result.map(Ingredient::Value))
            .unwrap_or(Ok(self))
    }
}

macro_rules! handle {
    ($result: expr) => {
        match $result {
            Some(Ok(value)) => value,
            Some(Err(error)) => return Some(Err(error)),
            None => return None,
        }
    };
}

impl Resolve for Method {
    fn solve(self, row: RecipeKey) -> RecipeSolution {
        Some(match self {
            Method::BooleanCheck(check, recipe) => check.solve(handle!(recipe.solve(row))),
            Method::UnaryOperation(operator, recipe) => operator.solve(handle!(recipe.solve(row))),
            Method::BinaryOperation(operator, left, right) => {
                operator.solve(handle!(left.solve(row)), handle!(right.solve(row)))
            }
            Method::Function(function, arguments) => {
                let arguments = handle!(arguments
                    .into_iter()
                    .map(|argument| argument.solve(row))
                    .collect::<Option<Result<Vec<Value>>>>());
                function.solve(arguments)
            }
            _ => unimplemented!(),
        })
    }
    fn simplify(self, row: RecipeKey) -> Result<Self> {
        Ok(match self {
            Method::Aggregate(aggregate, recipe) => {
                Method::Aggregate(aggregate, recipe.simplify(row)?)
            }
            Method::BinaryOperation(operator, left, right) => {
                let (left, right) = (left.simplify(row)?, right.simplify(row)?);
                if let (Some(left), Some(right)) = (left.as_solution(), right.as_solution()) {
                    Method::Value(operator.solve(left, right)?)
                } else {
                    Method::BinaryOperation(operator, left, right)
                }
            }
            _ => self,
        })
    }
}
