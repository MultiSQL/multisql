use {
    super::{Ingredient, Method, Recipe, RecipeError, RecipeSolution},
    crate::{executor::select::select, Result, Row, Value},
    futures::executor::block_on,
    sqlparser::ast::{Expr, Value as AstValue},
};

pub type ResolveKeys<'a> = Option<&'a Keys<'a>>;

pub struct Keys<'a> {
    pub row: Option<&'a Row>,
}

pub trait Resolve {
    fn solve(self, keys: ResolveKeys) -> RecipeSolution;
    fn simplify(self, keys: ResolveKeys) -> Result<Self>
    where
        Self: Sized;
}

impl Resolve for Recipe {
    fn solve(self, keys: ResolveKeys) -> RecipeSolution {
        match self {
            Recipe::Ingredient(ingredient) => ingredient.solve(keys),
            Recipe::Method(method) => method.solve(keys),
        }
    }
    fn simplify(self, keys: ResolveKeys) -> Result<Self> {
        match self {
            Recipe::Ingredient(ingredient) => ingredient.simplify(keys).map(Recipe::Ingredient),
            Recipe::Method(method) => method.simplify(keys).map(|method| {
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
    fn solve(self, keys: ResolveKeys) -> RecipeSolution {
        match self {
            Ingredient::Value(value) => Some(Ok(value)),
            Ingredient::Column(index) => keys
                .map(|keys| {
                    keys.row.clone().map(|row| {
                        row.get_value(index)
                            .ok_or(
                                RecipeError::Failed(String::from(
                                    "Couldn't get value with rows index!",
                                ))
                                .into(),
                            )
                            .map(|value| value.clone())
                    })
                })
                .flatten(),
        }
    }
    fn simplify(self, keys: ResolveKeys) -> Result<Self> {
        self.clone()
            .solve(keys)
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
    fn solve(self, keys: ResolveKeys) -> RecipeSolution {
        Some(match self {
            Method::Value(_) => unreachable!(),
            Method::BooleanCheck(check, recipe) => check.solve(handle!(recipe.solve(keys))),
            Method::UnaryOperation(operator, recipe) => operator.solve(handle!(recipe.solve(keys))),
            Method::BinaryOperation(operator, left, right) => {
                operator.solve(handle!(left.solve(keys)), handle!(right.solve(keys)))
            }
            Method::Function(function, arguments) => {
                let arguments = handle!(arguments
                    .into_iter()
                    .map(|argument| argument.solve(keys))
                    .collect::<Option<Result<Vec<Value>>>>());
                function.solve(arguments)
            }
            Method::Cast(data_type, recipe) => unimplemented!(),

            Method::Aggregate(aggregate, recipe) => unimplemented!(),
            Method::Subquery(query) => {
                return query.column.solve(keys);
            } // Subquery should (hopefully!) already be joined.
        })
    }
    fn simplify(self, keys: ResolveKeys) -> Result<Self> {
        Ok(match self {
            Method::Value(_) => return Err(RecipeError::Unreachable.into()),
            Method::BooleanCheck(check, recipe) => {
                Method::BooleanCheck(check, recipe.simplify(keys)?)
            }
            Method::UnaryOperation(operator, recipe) => {
                Method::UnaryOperation(operator, recipe.simplify(keys)?)
            }
            Method::BinaryOperation(operator, left, right) => {
                let (left, right) = (left.simplify(keys)?, right.simplify(keys)?);
                if let (Some(left), Some(right)) = (left.as_solution(), right.as_solution()) {
                    Method::Value(operator.solve(left, right)?)
                } else {
                    Method::BinaryOperation(operator, left, right)
                }
            }
            Method::Function(function, arguments) => {
                let arguments = arguments
                    .into_iter()
                    .map(|argument| argument.simplify(keys))
                    .collect::<Result<Vec<Recipe>>>()?;
                Method::Function(function, arguments)
            }
            Method::Cast(data_type, recipe) => Method::Cast(data_type, recipe.simplify(keys)?),

            Method::Aggregate(aggregate, recipe) => {
                Method::Aggregate(aggregate, recipe.simplify(keys)?)
            }
            Method::Subquery(mut query) => {
                query.column = query.column.simplify(keys)?;
                Method::Subquery(query)
            }
        })
    }
}
