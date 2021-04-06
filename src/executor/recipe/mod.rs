mod from;
mod method;
mod resolve;

use {
    crate::{Row, Value},
    method::{Aggregate, BinaryOperator, BooleanCheck, UnaryOperator},
    serde::Serialize,
    sqlparser::DataType,
    std::fmt::Debug,
    thiserror::Error,
};

enum Recipe {
    Ingredient(Ingredient),
    Method(Method),
}

enum Ingredient {
    Value(Value),
    Column(usize),
}

enum Method {
    Value(Value), // SIMPLIFICATION ONLY!

    BooleanCheck(BooleanCheck, Recipe),
    UnaryOperation(UnaryOperator, Recipe),
    BinaryOperation(BinaryOperator, Recipe, Recipe),
    Function(Function, Vec<Recipe>),

    Cast(DataType, Recipe),

    Aggregate(Aggregate, Recipe),
}

#[derive(Error, Serialize, Debug, PartialEq)]
enum RecipeError {
    #[error("recipe missing components")]
    MissingComponents,

    #[error("{0} is either invalid or unimplemented")]
    UnimplementedFunction(String),
}

type RecipeKey = Option<Row>;
type RecipeSolution = Option<Result<Value>>;

impl Recipe {
    fn as_solution(self, row: RecipeKey) -> Option<Value> {
        if let Recipe::Ingredient(Ingredient::Value(value)) = self {
            Some(value)
        } else {
            None
        }
    }
    fn must_solve(self, row: RecipeKey) -> Result<Value> {
        self.solve(row).or(Err(MissingComponents.into()))
    }
}
