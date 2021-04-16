mod from;
mod new;
mod planned;
mod resolve;

pub use {
    from::TryIntoMethod,
    new::MetaRecipe,
    planned::PlannedRecipe,
    resolve::{Resolve, SimplifyBy},
};

use {
    crate::{
        executor::types::{ComplexColumnName, ObjectName, Row},
        Result, Value,
    },
    serde::Serialize,
    sqlparser::ast::{DataType, Expr},
    std::fmt::Debug,
    thiserror::Error,
};

#[derive(Error, Serialize, Debug, PartialEq)]
pub enum RecipeError {
    #[error("recipe missing components")]
    MissingComponents,

    #[error("{0} is either invalid or unimplemented")]
    InvalidQuery(String),
    #[error("{0} is invalid or unimplemented")]
    InvalidExpression(Expr),
    #[error("a function is either invalid or unimplemented")]
    InvalidFunction,

    #[error("column '{0:?}' could not be found")]
    MissingColumn(ObjectName),
    #[error("column '{0:?}' could mean various different columns, please be more specific with (table).(column)")]
    AmbiguousColumn(ObjectName),

    #[error("{0} is either invalid or unimplemented")]
    UnimplementedQuery(String),
    #[error("{0} is either invalid or unimplemented")]
    UnimplementedMethod(String),
    #[error("{0} is unimplemented")]
    UnimplementedExpression(Expr),

    #[error("other failure occurred: {0}")]
    Failed(String),

    #[error("this should be impossible, please report")]
    UnreachableNotAggregate,
    #[error("this should be impossible, please report")]
    Unreachable,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Recipe {
    Ingredient(Ingredient),
    Method(Box<Method>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Ingredient {
    Value(Value),
    Column(usize),
    Aggregate(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Method {
    Value(Value), // Only occurs backwards for eval! Should never be returned outside of a recursive simplification!
    Aggregate(AggregateOperator, Recipe), // Only occurs inside Ingredient::Aggregate. Perhaps this should not be a Method.

    UnaryOperation(UnaryOperator, Recipe),
    BinaryOperation(BinaryOperator, Recipe, Recipe),
    Function(FunctionOperator, Vec<Recipe>),

    Cast(DataType, Recipe),
}

// Cannot derive Debug for references. Perhaps these shouldn't consume their operators. TODO.
pub type UnaryOperator = fn(Value) -> Result<Value>;
pub type BinaryOperator = fn(Value, Value) -> Result<Value>;
pub type FunctionOperator = fn(Vec<Value>) -> Result<Value>;
pub type AggregateOperator = fn(Value, Value) -> Result<Value>;

pub trait RecipeUtilities {
    fn as_solution(&self) -> Option<Value>;
}
impl RecipeUtilities for Recipe {
    fn as_solution(&self) -> Option<Value> {
        if let Recipe::Ingredient(Ingredient::Value(value)) = self {
            Some(value.clone())
        } else {
            None
        }
    }
}
impl RecipeUtilities for MetaRecipe {
    fn as_solution(&self) -> Option<Value> {
        self.recipe.as_solution()
    }
}
impl RecipeUtilities for PlannedRecipe {
    fn as_solution(&self) -> Option<Value> {
        self.recipe.as_solution()
    }
}

impl Recipe {
    pub const NULL: Recipe = Recipe::Ingredient(Ingredient::Value(Value::Null));
    pub const TRUE: Recipe = Recipe::Ingredient(Ingredient::Value(Value::Bool(true)));
    pub const SINGLE_COLUMN: Recipe = Recipe::Ingredient(Ingredient::Column(0));
}
