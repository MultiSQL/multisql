use {
    crate::{
        data::{LiteralError, RowError, TableError, ValueError},
        executor::{
            AggregateError, AlterError, BlendError, EvaluateError, ExecuteError, FetchError,
            FilterError, JoinError, LimitError, RecipeError, SelectError, UpdateError,
            ValidateError,
        },
    },
    serde::Serialize,
    thiserror::Error as ThisError,
};

#[cfg(feature = "alter-table")]
use crate::store::AlterTableError;

#[derive(ThisError, Serialize, Debug)]
pub enum Error {
    #[cfg(feature = "alter-table")]
    #[error(transparent)]
    AlterTable(#[from] AlterTableError),

    #[error(transparent)]
    #[serde(with = "stringify")]
    Storage(#[from] Box<dyn std::error::Error>),

    #[error(transparent)]
    Execute(#[from] ExecuteError),
    #[error(transparent)]
    Alter(#[from] AlterError),
    #[error(transparent)]
    Fetch(#[from] FetchError),
    #[error(transparent)]
    Evaluate(#[from] EvaluateError),
    #[error(transparent)]
    Select(#[from] SelectError),
    #[error(transparent)]
    Join(#[from] JoinError),
    #[error(transparent)]
    Blend(#[from] BlendError),
    #[error(transparent)]
    Aggregate(#[from] AggregateError),
    #[error(transparent)]
    Update(#[from] UpdateError),
    #[error(transparent)]
    Filter(#[from] FilterError),
    #[error(transparent)]
    Limit(#[from] LimitError),
    #[error(transparent)]
    Row(#[from] RowError),
    #[error(transparent)]
    Table(#[from] TableError),
    #[error(transparent)]
    Validate(#[from] ValidateError),
    #[error(transparent)]
    Value(#[from] ValueError),
    #[error(transparent)]
    Literal(#[from] LiteralError),
    #[error(transparent)]
    Recipe(#[from] RecipeError),
}

pub type Result<T> = std::result::Result<T, Error>;
pub type MutResult<T, U> = std::result::Result<(T, U), (T, Error)>;

impl PartialEq for Error {
    fn eq(&self, other: &Error) -> bool {
        use Error::*;

        match (self, other) {
            #[cfg(feature = "alter-table")]
            (AlterTable(l), AlterTable(r)) => l == r,
            (Execute(l), Execute(r)) => l == r,
            (Alter(l), Alter(r)) => l == r,
            (Fetch(l), Fetch(r)) => l == r,
            (Evaluate(l), Evaluate(r)) => l == r,
            (Select(l), Select(r)) => l == r,
            (Join(l), Join(r)) => l == r,
            (Blend(l), Blend(r)) => l == r,
            (Aggregate(l), Aggregate(r)) => l == r,
            (Update(l), Update(r)) => l == r,
            (Filter(l), Filter(r)) => l == r,
            (Limit(l), Limit(r)) => l == r,
            (Row(l), Row(r)) => l == r,
            (Table(l), Table(r)) => l == r,
            (Validate(l), Validate(r)) => l == r,
            (Value(l), Value(r)) => l == r,
            (Literal(l), Literal(r)) => l == r,
            (Recipe(l), Recipe(r)) => l == r,
            _ => false,
        }
    }
}

mod stringify {
    use serde::Serializer;
    use std::fmt::Display;

    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Display,
        S: Serializer,
    {
        serializer.collect_str(value)
    }
}
