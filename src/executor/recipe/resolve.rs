use {
    super::{Ingredient, Method, Recipe},
    crate::{Row, Value},
    serde::Serialize,
    sqlparser::DataType,
    std::{cmp::min, fmt::Debug},
    thiserror::Error,
};

trait Resolve {
    fn solve(self, row: RecipeKey) -> RecipeSolution;
    fn simplify(self, row: RecipeKey) -> Result<Self>;
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
            Recipe::Ingredient(ingredient) => Recipe::Ingredient(ingredient.simplify(row)),
            Recipe::Method(method) => method.simplify(row).map(|method| {
                if let Method::Value(value) = method {
                    Recipe::Ingredient(Ingredient::Value(value))
                } else {
                    Recipe::Method(method)
                }
            }),
        }
    }
}

impl Resolve for Ingredient {
    fn solve(self, row: RecipeKey) -> RecipeSolution {
        match self {
            Ingredient::Value(value) => value,
            Ingredient::Column(index) => row.map(row.get(index)),
        }
    }
    fn simplify(self, row: RecipeKey) -> Result<Self> {
        self.solve(row)
            .map(|result| Ingredient::Value(result?))
            .or(self);
    }
}

impl Resolve for Method {
    fn solve(self, row: RecipeKey) -> RecipeSolution {
        match self {
            Method::BooleanCheck(check, recipe) => check.solve(recipe.solve(row)??),
            Method::UnaryOperation(operator, recipe) => operator.solve(recipe.solve(row)??),
            Method::BinaryOperation(operator, left, right) => {
                operator.solve(left.solve(row)??, right.solve(row)??)
            }
            Method::Function(function, arguments) => {
                let arguments = arguments.into_iter().map(|argument| argument.solve(row));
                if let Some(issue) =
                    arguments.find(|argument| matches!(argument, None | Some(Err(_))))
                {
                    issue
                } else {
                    function(function, arguments.collect())
                }
            }
            _ => unimplemented!(),
        }
    }
    fn simplify(self, row: RecipeKey) -> Result<Self> {
        match self {
            Method::Aggregate(aggregate, recipe) => Method::Aggregate(aggregate, recipe.simplify()),
            Method::BinaryOperation(operator, left, right) => {
                let (left, right) = (left.simplify(row)?, right.simplify(row)?);
                if let (Some(left), Some(right)) = (left.as_solution(), right.as_solution()) {
                    Method::Value(operator.solve(left, right))
                } else {
                    Method::BinaryOperation(operator, left, right)
                }
            }
            _ => self,
        }
    }
}
