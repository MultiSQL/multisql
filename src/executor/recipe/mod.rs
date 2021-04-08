mod from;
pub mod manual;
mod method;
mod resolve;

pub use {
    manual::{Join, Manual},
    method::CalculationError,
    resolve::Resolve,
};

use {
    crate::{Result, Row, Value},
    method::{Aggregate, BinaryOperator, BooleanCheck, Function, UnaryOperator},
    serde::Serialize,
    sqlparser::ast::{DataType, Expr},
    std::fmt::Debug,
    thiserror::Error,
};

#[derive(Debug, Clone)]
pub enum Recipe {
    Ingredient(Ingredient),
    Method(Box<Method>),
}

#[derive(Debug, Clone)]
pub enum Ingredient {
    Value(Value),
    Column(usize),
}

#[derive(Debug, Clone)]
pub enum Method {
    Value(Value), // SIMPLIFICATION ONLY!

    BooleanCheck(BooleanCheck, Recipe),
    UnaryOperation(UnaryOperator, Recipe),
    BinaryOperation(BinaryOperator, Recipe, Recipe),
    Function(Function, Vec<Recipe>),

    Cast(DataType, Recipe),

    Aggregate(Aggregate, Recipe),
}

#[derive(Error, Serialize, Debug, PartialEq)]
pub enum RecipeError {
    #[error("recipe missing components")]
    MissingComponents,

    #[error("{0} is either invalid or unimplemented")]
    InvalidQuery(String),
    #[error("a function is either invalid or unimplemented")]
    InvalidFunction,

    #[error("{0} is either invalid or unimplemented")]
    UnimplementedQuery(String),
    #[error("{0} is either invalid or unimplemented")]
    UnimplementedMethod(String),
    #[error("{0} is unimplemented")]
    UnimplementedExpression(Expr),

    #[error("other failure occurred: {0}")]
    Failed(String),
}

type RecipeKey<'a> = Option<&'a Row>;
type RecipeSolution = Option<Result<Value>>;
type MethodRecipeSolution = Result<Value>;

impl Recipe {
    pub fn as_solution(&self) -> Option<Value> {
        if let Recipe::Ingredient(Ingredient::Value(value)) = self {
            Some(value.clone())
        } else {
            None
        }
    }
    pub fn must_solve(self, row: &Row) -> Result<Value> {
        self.solve(Some(row))
            .unwrap_or(Err(RecipeError::MissingComponents.into()))
    }
    pub fn confirm(self, row: &Row) -> Result<bool> {
        Ok(matches!(
            self.must_solve(row)?,
            Value::Null | Value::Bool(true)
        ))
    }
}
