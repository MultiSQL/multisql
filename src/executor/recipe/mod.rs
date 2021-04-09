mod find;
mod from;
mod macro_components;
pub mod manual;
pub mod method;
mod resolve;

pub use {
    find::Find,
    macro_components::{MacroComponents, Subquery},
    manual::{Join, Manual},
    method::CalculationError,
    resolve::{Keys, Resolve, ResolveKeys},
};

use {
    crate::{Result, Row, Store, Value},
    method::{Aggregate, BinaryOperator, BooleanCheck, Function, UnaryOperator},
    serde::Serialize,
    sqlparser::ast::{DataType, Expr},
    std::fmt::Debug,
    thiserror::Error,
};

const RECIPE_NULL: Recipe = Recipe::Ingredient(Ingredient::Value(Value::Null));

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
    Subquery(Subquery),
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

    #[error("this should be impossible, please report")]
    Unreachable,
}

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
    pub fn must_solve<'a>(self, row: &'a Row) -> Result<Value> {
        println!("{:?}", self.clone());
        self.solve(Some(&Keys { row: Some(row) }))
            .unwrap_or(Err(RecipeError::MissingComponents.into()))
    }
    pub fn confirm<'a>(self, row: &'a Row) -> Result<bool> {
        Ok(matches!(
            self.must_solve(row)?,
            Value::Null | Value::Bool(true)
        ))
    }
}
