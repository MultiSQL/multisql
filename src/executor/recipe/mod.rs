mod from;
mod manual;
mod method;
mod resolve;

pub use {manual::Manual, resolve::Resolve};

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
    UnimplementedMethod(String),

    #[error("{0} is unimplemented")]
    UnimplementedExpression(Expr),

    #[error(
        "number of function parameters not matching for function: {function:?}; expected: {expected:?}, found: {found:?}"
    )]
    WrongNumberOfArguments {
        function: Function,
        expected: usize,
        found: usize,
    },

    #[error(
        "data types for function: {function:?} wrong, expected: {expected:?}, found: {found:?}"
    )]
    FunctionRequiresDataType {
        function: Function,
        expected: Value,
        found: Value,
    },

    #[error("function: {0:?} failed: {1}")]
    FailedFunction(Function, String),

    #[error("other failure occurred: {0}")]
    Failed(String),
}

type RecipeKey = &'static Option<Row>;
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
    pub fn must_solve(self, row: RecipeKey) -> Result<Value> {
        self.solve(row)
            .unwrap_or(Err(RecipeError::MissingComponents.into()))
    }
}
