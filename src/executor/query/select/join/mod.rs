mod execute;
mod manual;
mod method;
mod plan;

pub use {execute::JoinExecute, manual::JoinManual, method::JoinMethod, plan::JoinPlan};

use {
    crate::{executor::types::Row, Value},
    serde::Serialize,
    std::fmt::Debug,
    thiserror::Error as ThisError,
};

#[derive(ThisError, Serialize, Debug, PartialEq)]
pub enum JoinError {
    #[error("join type not yet implemented")]
    UnimplementedJoinType,
    #[error("join constraint not yet implemented")]
    UnimplementedJoinConstaint,
    #[error("table type not yet implemented")]
    UnimplementedTableType,
    #[error("amount of components in identifier not yet supported")]
    UnimplementedNumberOfComponents,

    #[error("this should be impossible, please report")]
    UnreachableCellNotFound,
    #[error("this should be impossible, please report")]
    Unreachable,
}

#[derive(Ord, Eq, PartialEq, PartialOrd, Debug, Clone)]
pub enum JoinType {
    Inner, // Reduces rows so go first
    Left,
    Right,
    Full,
    CrossJoin, // All join: NO FILTER
}

impl JoinType {
    pub fn includes_left(&self) -> bool {
        matches!(self, JoinType::Left | JoinType::Full)
    }
    pub fn includes_right(&self) -> bool {
        matches!(self, JoinType::Right | JoinType::Full)
    }
}
