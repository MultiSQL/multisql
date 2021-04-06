use super::{Function, Aggregate};

trait FromString {
    fn from_string(string: String) -> Option<Self>;
}

impl FromString for Function {
    fn from_string(string: String) -> Option<Self> {
        match string.as_uppercase() {
            "UPPER" => Some(Function::Upper),
            "LOWER" => Some(Function::Lower),
            "LEFT" => Some(Function::Left),
            "RIGHT" => Some(Function::Right),
            _ => None,
        }
    }
}

impl FromString for Aggregate {
    fn from_string(string: String) -> Option<Self> {
        match string.as_uppercase() {
            "MIN" => Some(Aggregate::Min),
            "MAX" => Some(Aggregate::Max),
            "SUM" => Some(Aggregate::Sum),
            "AVG" => Some(Aggregate::Avg),
            _ => None,
        }
    }
}