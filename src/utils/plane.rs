use crate::{Row, Value};

pub(crate) type KeyedRow = (Value, Row);
pub(crate) type Plane = Vec<KeyedRow>;
