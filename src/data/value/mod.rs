use {
	crate::result::Result,
	sqlparser::ast::DataType,
	std::{
		cmp::Ordering,
		hash::{Hash, Hasher},
	},
};

mod big_endian;
mod cast;
mod convert;
mod declare;
mod error;
mod literal;
mod methods;
mod serde_convert;
mod value_type;

pub use {
	big_endian::BigEndian,
	cast::{Cast, CastWithRules},
	convert::{Convert, ConvertFrom},
	declare::{Null, Value},
	error::ValueError,
	value_type::ValueType,
};

/// # Value
/// Value is MultiSQL's value wrapper and stores any values which interact with the stores.
/// At times they may be converted in the interface for convinence but otherwise, all value interactions with MultiSQL require this wrapper.
///
/// ## Conversion
/// Value implements conversion from inner types; for example:
///
/// ```
/// # use multisql::Value;
/// let value: Value = Value::I64(10);
/// let int: i64 = 10;
///
/// let int_value: Value = int.into();
///
/// assert_eq!(value, int_value);
/// ```
///
/// ### Casting
/// Values can be cast between types via [Cast], for example:
///
/// ```
/// # use multisql::{Value, Cast};
/// let value_str: Value = Value::Str(String::from("10"));
/// let int: i64 = 10;
///
/// let str_int: i64 = value_str.cast().unwrap();
///
/// assert_eq!(int, str_int);
///
/// assert_eq!(Value::I64(int), Value::I64(Value::Str(String::from("10")).cast().unwrap()));
/// ```
///
/// ## Equality
/// Values of the same type compare as their inner values would.
///
/// Null never equals Null.
///
/// Floats and Integers implicitly compare and convert.
/// (Feature: `implicit_float_conversion`)

impl Hash for Value {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.to_be_bytes().hash(state)
	}
}
impl Ord for Value {
	fn cmp(&self, other: &Self) -> Ordering {
		self.partial_cmp(other).unwrap_or(Ordering::Equal)
	}
}

pub trait NullOrd {
	fn null_cmp(&self, other: &Self) -> Option<Ordering>;
}

impl NullOrd for Value {
	fn null_cmp(&self, other: &Self) -> Option<Ordering> {
		self.partial_cmp(other).or(match (self, other) {
			(Value::Null(_), Value::Null(_)) => None,
			(Value::Null(_), _) => Some(Ordering::Less),
			(_, Value::Null(_)) => Some(Ordering::Greater),
			_ => None,
		})
	}
}

impl Value {
	pub fn validate_type(mut self, data_type: &DataType) -> Result<Self> {
		let mut valid = self.type_is_valid(data_type);

		if !valid {
			let converted = match data_type {
				DataType::Float(_) => self.clone().convert().map(Value::F64).ok(),
				_ => None,
			};
			if let Some(converted) = converted {
				if converted.type_is_valid(data_type) {
					valid = true;
					self = converted;
				}
			}
		}

		if !valid {
			return Err(ValueError::IncompatibleDataType {
				data_type: data_type.to_string(),
				value: format!("{:?}", self),
			}
			.into());
		}

		Ok(self)
	}
	pub fn is(&mut self, data_type: &ValueType) -> Result<()> {
		match (data_type, &self) {
			(ValueType::Bool, Value::Bool(_))
			| (ValueType::U64, Value::U64(_))
			| (ValueType::I64, Value::I64(_))
			| (ValueType::F64, Value::F64(_))
			| (ValueType::Str, Value::Str(_))
			| (ValueType::Timestamp, Value::Timestamp(_))
			| (ValueType::Any, _)
			| (_, Value::Null(_)) => Ok(()),
			(ValueType::F64, Value::I64(_)) => {
				*self = Value::F64(self.clone().cast()?);
				Ok(())
			}
			_ => Err(ValueError::IncompatibleDataType {
				data_type: data_type.to_string(),
				value: format!("{:?}", self),
			}
			.into()),
		}
	}

	fn type_is_valid(&self, data_type: &DataType) -> bool {
		matches!(
			(data_type, self),
			(DataType::Boolean, Value::Bool(_))
				| (DataType::Int(_), Value::I64(_))
				| (DataType::Float(_), Value::F64(_))
				| (DataType::Text, Value::Str(_))
				| (DataType::Boolean, Value::Null(_))
				| (DataType::Int(_), Value::Null(_))
				| (DataType::Float(_), Value::Null(_))
				| (DataType::Text, Value::Null(_))
		)
	}

	pub fn validate_null(&self, nullable: bool) -> Result<()> {
		if !nullable && matches!(self, Value::Null(_)) {
			return Err(ValueError::NullValueOnNotNullField.into());
		}

		Ok(())
	}

	pub fn cast_datatype(&self, data_type: &DataType) -> Result<Self> {
		match (data_type, self) {
			(DataType::Boolean, Value::Bool(_))
			| (DataType::Int(_), Value::I64(_))
			| (DataType::Float(_), Value::F64(_))
			| (DataType::Text, Value::Str(_)) => Ok(self.clone()),
			(_, Value::Null(_)) => Ok(Value::Null),

			(DataType::Boolean, value) => value.clone().cast().map(Value::Bool),
			(DataType::Int(_), value) => value.clone().cast().map(Value::I64),
			(DataType::Float(_), value) => value.clone().cast().map(Value::F64),
			(DataType::Text, value) => value.clone().cast().map(Value::Str),

			_ => Err(ValueError::UnimplementedCast.into()),
		}
	}

	pub fn inc(&self) -> Self {
		match self {
			Value::Bool(false) => Value::Bool(true),
			Value::I64(val) => Value::I64(val + 1),
			Value::F64(val) => Value::F64(f64::from_bits(val.to_bits() + 1)),
			_ => unimplemented!(), // TODO: Handle better & expand
		}
	}
	pub fn dec(&self) -> Self {
		match self {
			Value::Bool(true) => Value::Bool(false),
			Value::I64(val) => Value::I64(val - 1),
			Value::F64(val) => Value::F64(f64::from_bits(val.to_bits() - 1)),
			_ => unimplemented!(), // TODO: Handle better & expand
		}
	}
}
