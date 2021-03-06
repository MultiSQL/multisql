use {
	crate::result::Result,
	serde::{Deserialize, Serialize},
	sqlparser::ast::DataType,
	std::{
		cmp::Ordering,
		fmt::Debug,
		hash::{Hash, Hasher},
	},
};

mod big_endian;
mod cast;
mod convert;
mod error;
mod literal;
mod methods;
mod serde_convert;
mod value_type;

pub use {
	big_endian::BigEndian,
	cast::{Cast, CastWithRules},
	convert::{Convert, ConvertFrom},
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
	Null,

	Bool(bool),
	U64(u64),
	I64(i64),
	F64(f64),
	Str(String),

	Bytes(Vec<u8>),
	Timestamp(i64),

	Internal(i64),
}

impl Hash for Value {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.to_be_bytes().hash(state)
	}
}
impl Eq for Value {}
impl Ord for Value {
	fn cmp(&self, other: &Self) -> Ordering {
		self.partial_cmp(other).unwrap_or(Ordering::Equal)
	}
}

impl From<bool> for Value {
	fn from(from: bool) -> Value {
		Value::Bool(from)
	}
}
impl From<i64> for Value {
	fn from(from: i64) -> Value {
		Value::I64(from)
	}
}
impl From<i32> for Value {
	fn from(from: i32) -> Value {
		Value::I64(from as i64)
	}
}
impl From<u64> for Value {
	fn from(from: u64) -> Value {
		Value::U64(from)
	}
}
impl From<u32> for Value {
	fn from(from: u32) -> Value {
		Value::U64(from as u64)
	}
}
impl From<f64> for Value {
	fn from(from: f64) -> Value {
		Value::F64(from)
	}
}
impl From<String> for Value {
	fn from(from: String) -> Value {
		Value::Str(from)
	}
}

impl<T: Into<Value>> From<Option<T>> for Value {
	fn from(from: Option<T>) -> Value {
		from.map(|v| v.into()).unwrap_or(Value::Null)
	}
}

impl From<Value> for String {
	// assumes all can be cast to string
	fn from(from: Value) -> String {
		from.cast().unwrap()
	}
}

impl PartialEq for Value {
	fn eq(&self, other: &Value) -> bool {
		match (self, other) {
			(Value::Bool(l), Value::Bool(r)) => l == r,
			(Value::U64(l), Value::U64(r)) => l == r,
			(Value::I64(l), Value::I64(r)) => l == r,
			(Value::F64(l), Value::F64(r)) => l == r,
			(Value::Str(l), Value::Str(r)) => l == r,
			(Value::Bytes(l), Value::Bytes(r)) => l == r,
			(Value::Timestamp(l), Value::Timestamp(r)) => l == r,

			(Value::Internal(l), Value::Internal(r)) => l == r,

			#[cfg(feature = "implicit_float_conversion")]
			(Value::I64(l), Value::F64(r)) => (*l as f64) == *r,
			#[cfg(feature = "implicit_float_conversion")]
			(Value::F64(l), Value::I64(r)) => *l == (*r as f64),
			_ => false,
		}
	}
}

impl PartialOrd for Value {
	fn partial_cmp(&self, other: &Value) -> Option<Ordering> {
		match (self, other) {
			(Value::Bool(l), Value::Bool(r)) => Some(l.cmp(r)),
			(Value::I64(l), Value::I64(r)) => Some(l.cmp(r)),
			(Value::F64(l), Value::F64(r)) => l.partial_cmp(r),
			(Value::Str(l), Value::Str(r)) => Some(l.cmp(r)),
			(Value::Bytes(l), Value::Bytes(r)) => Some(l.cmp(r)),
			(Value::Timestamp(l), Value::Timestamp(r)) => Some(l.cmp(r)),

			(Value::Internal(l), Value::Internal(r)) => Some(l.cmp(r)),

			#[cfg(feature = "implicit_float_conversion")]
			(Value::I64(l), Value::F64(r)) => (*l as f64).partial_cmp(r),

			#[cfg(feature = "implicit_float_conversion")]
			(Value::F64(l), Value::I64(r)) => l.partial_cmp(&(*r as f64)),

			_ => None,
		}
	}
}

pub trait NullOrd {
	fn null_cmp(&self, other: &Self) -> Option<Ordering>;
}

impl NullOrd for Value {
	fn null_cmp(&self, other: &Self) -> Option<Ordering> {
		self.partial_cmp(other).or(match (self, other) {
			(Value::Null, Value::Null) => None,
			(Value::Null, _) => Some(Ordering::Less),
			(_, Value::Null) => Some(Ordering::Greater),
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
			| (_, Value::Null) => Ok(()),
			(ValueType::Timestamp, Value::I64(val)) => {
				*self = Value::Timestamp(*val);
				Ok(())
			}
			(ValueType::I64, Value::Timestamp(val)) => {
				*self = Value::I64(*val);
				Ok(())
			}
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
				| (DataType::Boolean, Value::Null)
				| (DataType::Int(_), Value::Null)
				| (DataType::Float(_), Value::Null)
				| (DataType::Text, Value::Null)
		)
	}

	pub fn validate_null(&self, nullable: bool) -> Result<()> {
		if !nullable && matches!(self, Value::Null) {
			return Err(ValueError::NullValueOnNotNullField.into());
		}

		Ok(())
	}

	pub fn cast_datatype(&self, data_type: &DataType) -> Result<Self> {
		self.cast_valuetype(&data_type.clone().into())
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
