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
    #[error("table type not yet implemented")]
    UnimplementedTableType,

    #[error("this should be impossible, please report")]
    UnreachableCellNotFound,
    #[error("this should be impossible, please report")]
    Unreachable,
}

type ColumnIndexes = Vec<(usize, Vec<usize>)>;
type ColumnValues = Vec<(Value, Vec<Value>)>;

#[derive(Ord, Eq, PartialEq, PartialOrd, Debug, Clone)]
pub enum JoinType {
    Inner, // Reduces rows so go first
    Left,
    Right,
    Full,      // Not currently implemented!
    CrossJoin, // Full join; NO FILTER
}

impl JoinType {
    pub fn complete_join(&self, mut plane_row: Row, self_row: Row) -> Vec<Row> {
        plane_row.extend(self_row);
        vec![plane_row]
    }
    pub fn incomplete_join(&self, mut plane_row: Row, join_row: Row) -> Vec<Row> {
        match self {
            // I want to find a way to get rid of this match, I want to be able to call directly based on JoinType.
            // See: https://github.com/rust-lang/rfcs/pull/1450
            // Returning Vec is problematic, we should be able to handle based on JoinType of a Join, this adds overheads of making Vec.
            // This is a crucial, very high use function, performance matters here.
            JoinType::Inner => vec![],
            JoinType::Left => {
                plane_row.extend(vec![Value::Null; join_row.len()]);
                vec![plane_row]
            }
            JoinType::Right => {
                let mut plane_row = vec![Value::Null; plane_row.len()];
                plane_row.extend(join_row);
                vec![plane_row]
            }
            JoinType::Full => {
                let mut right_plane_row = vec![Value::Null; plane_row.len()];
                let left_plane_extension = vec![Value::Null; join_row.len()];
                right_plane_row.extend(join_row);
                plane_row.extend(left_plane_extension);
                vec![plane_row, right_plane_row]
            }
            JoinType::CrossJoin => unreachable!(),
        }
    }
}
