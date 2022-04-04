use {
	super::{error::ValueError, Value},
	crate::result::{Error, Result},
	sqlparser::ast::Value as AstValue,
	std::convert::TryFrom,
};

impl<'a> TryFrom<&'a AstValue> for Value {
	type Error = Error;

	fn try_from(ast_value: &'a AstValue) -> Result<Self> {
		match ast_value {
			AstValue::Boolean(value) => Ok(Value::Bool(*value)),
			AstValue::Number(value, false) => value
				.parse::<i64>()
				.map_or_else(
					|_| value.parse::<f64>().map(Value::F64),
					|value| Ok(Value::I64(value)),
				)
				.map_err(|_| ValueError::FailedToParseNumber.into()),
			AstValue::SingleQuotedString(value) => Ok(Value::Str(value.clone())),
			AstValue::Null => Ok(Value::Null),
			_ => Err(ValueError::UnimplementedLiteralType.into()),
		}
	}
}
