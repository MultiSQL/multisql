use {
	crate::{Cast, CastWithRules, Result, Value, ValueError, ValueType},
	chrono::NaiveDateTime,
	std::string::ToString,
};

impl Value {
	pub fn cast_valuetype(&self, value_type: &ValueType) -> Result<Self> {
		match (value_type, self) {
			(ValueType::Bool, Value::Bool(_))
			| (ValueType::I64, Value::I64(_))
			| (ValueType::F64, Value::F64(_))
			| (ValueType::Str, Value::Str(_))
			| (ValueType::Any, _) => Ok(self.clone()),
			(_, Value::Null) => Ok(Value::Null),

			(ValueType::Bool, value) => value.clone().cast().map(Value::Bool),
			(ValueType::I64, value) => value.clone().cast().map(Value::I64),
			(ValueType::F64, value) => value.clone().cast().map(Value::F64),
			(ValueType::Str, value) => value.clone().cast().map(Value::Str),
			(ValueType::Timestamp, value) => {
				let datetime: NaiveDateTime = value.clone().cast_with_rule(Value::Null)?;
				let timestamp = datetime.timestamp();
				Ok(Value::Timestamp(timestamp))
			}

			_ => Err(ValueError::UnimplementedCast.into()),
		}
	}
}

impl ToString for ValueType {
	fn to_string(&self) -> String {
		use ValueType::*;
		match self {
			Bool => String::from("Bool"),
			U64 => String::from("UInt"),
			I64 => String::from("Int"),
			F64 => String::from("Float"),
			Str => String::from("Text"),
			Timestamp => String::from("Timestamp"),
			Any => String::from("Any"),
		}
	}
}
