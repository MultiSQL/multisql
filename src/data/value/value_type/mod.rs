mod cast;

use {
	crate::Value,
	serde::{Deserialize, Serialize},
	sqlparser::ast::DataType,
	std::fmt::Debug,
};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum ValueType {
	Bool,
	U64,
	I64,
	F64,
	Str,
	Timestamp,
	Any,
}
impl Default for ValueType {
	fn default() -> Self {
		Self::Any
	}
}
impl From<Value> for ValueType {
	fn from(value: Value) -> Self {
		match value {
			Value::Bool(_) => ValueType::Bool,
			Value::U64(_) => ValueType::U64,
			Value::I64(_) => ValueType::I64,
			Value::F64(_) => ValueType::F64,
			Value::Str(_) => ValueType::Str,
			Value::Timestamp(_) => ValueType::Timestamp,
			_ => ValueType::Any,
		}
	}
}
impl From<DataType> for ValueType {
	fn from(data_type: DataType) -> Self {
		match data_type {
			DataType::Boolean => ValueType::Bool,
			DataType::UnsignedInt(_) => ValueType::U64,
			DataType::Int(_) => ValueType::I64,
			DataType::Float(_) => ValueType::F64,
			DataType::Text => ValueType::Str,
			DataType::Timestamp => ValueType::Timestamp,
			_ => ValueType::Any,
		}
	}
}
