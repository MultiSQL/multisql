use crate::{Cast, Value};

impl From<Value> for serde_json::value::Value {
	fn from(value: Value) -> serde_json::value::Value {
		match value {
			Value::Bool(value) => value.into(),
			Value::U64(value) => value.into(),
			Value::I64(value) => value.into(),
			Value::F64(value) => value.into(),
			Value::Str(value) => value.into(),
			Value::Null(_) => serde_json::value::Value::Null,
			other => {
				let string: String = other.cast().unwrap();
				string.into()
			}
		}
	}
}
