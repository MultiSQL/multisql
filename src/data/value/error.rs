use {
	crate::{Value, ValueType},
	serde::Serialize,
	std::fmt::Debug,
	thiserror::Error,
};

#[derive(Error, Serialize, Debug, PartialEq)]
pub enum ValueError {
	#[error("literal: {literal} is incompatible with data type: {data_type}")]
	IncompatibleLiteralForDataType { data_type: String, literal: String },

	#[error("incompatible data type, data type: {data_type}, value: {value}")]
	IncompatibleDataType { data_type: String, value: String },

	#[error("null value on not null field")]
	NullValueOnNotNullField,

	#[error("failed to parse number")]
	FailedToParseNumber,

	#[error("unreachable failure on parsing number")]
	UnreachableNumberParsing,

	#[error("floating columns cannot be set to unique constraint")]
	ConflictOnFloatWithUniqueConstraint,

	#[error(
		"number of function parameters not matching (expected: {expected:?}, found: {found:?})"
	)]
	NumberOfFunctionParamsNotMatching { expected: usize, found: usize },

	#[error("conversion rule is not accepted for this type")]
	InvalidConversionRule,

	#[error("impossible cast")]
	ImpossibleCast, // Bad error-- phase out
	#[error("failed to cast {0:?} into {1:?}")]
	FailedCast(Value, ValueType),

	#[error("date time failed to parse: {0}")]
	DateTimeParseError(String),
	#[error("failed to parse {0:?} as {1}")]
	ParseError(Value, &'static str),
	#[error("something went wrong with date math")]
	DateError, // Should avoid throwing
	#[error("timestamp error: {0}")]
	SpecifiedTimestampError(String), // Should avoid throwing

	#[error("cannot convert {0:?} into {1}")]
	CannotConvert(Value, &'static str),

	#[error("{1} only supports numeric values, found {0:?}")]
	OnlySupportsNumeric(Value, &'static str),
	#[error("{1} only supports boolean values, found {0:?}")]
	OnlySupportsBoolean(Value, &'static str),
	#[error("bad input: {0:?}")]
	BadInput(Value),

	#[error("unimplemented literal type")]
	UnimplementedLiteralType,
	#[error("unimplemented cast")]
	UnimplementedCast,
	#[error("unimplemented convert")]
	UnimplementedConvert,
	#[error("unreachable literal cast from number to integer: {0}")]
	UnreachableLiteralCastFromNumberToInteger(String),
	#[error("unimplemented literal cast: {literal} as {data_type}")]
	UnimplementedLiteralCast { data_type: String, literal: String },
}
