#![cfg(feature = "expanded-api")]
use crate::Value;

impl From<Value> for serde_json::value::Value {
	fn from(value: Value) -> serde_json::value::Value {
		match value {
			Value::Bool(value) => value.into(),
			Value::I64(value) => value.into(),
			Value::F64(value) => value.into(),
			Value::Str(value) => value.into(),
			Value::Null => serde_json::value::Value::Null,
			_ => unimplemented!(),
		}
	}
}
