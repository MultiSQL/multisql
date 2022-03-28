use {
	crate::result::Result,
	regex::Regex,
	serde::{Deserialize, Serialize},
	sqlparser::ast::DataType,
	std::{cmp::Ordering, fmt::Debug},
};

mod big_endian;
mod cast;
mod convert;
mod error;
mod literal;
mod methods;

pub use {
	big_endian::BigEndian,
	cast::{Cast, CastWithRules},
	convert::{Convert, ConvertFrom},
	error::ValueError,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
	Bool(bool),
	Bytes(Vec<u8>),
	I64(i64),
	F64(f64),
	Str(String),
	Null,
	Timestamp(i64),
	Internal(i64),
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

impl PartialEq for Value {
	fn eq(&self, other: &Value) -> bool {
		match (self, other) {
			(Value::Bool(l), Value::Bool(r)) => l == r,
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
		match (data_type, self) {
			(DataType::Boolean, Value::Bool(_))
			| (DataType::Int(_), Value::I64(_))
			| (DataType::Float(_), Value::F64(_))
			| (DataType::Text, Value::Str(_)) => Ok(self.clone()),
			(_, Value::Null) => Ok(Value::Null),

			(DataType::Boolean, value) => value.clone().cast().map(Value::Bool),
			(DataType::Int(_), value) => value.clone().cast().map(Value::I64),
			(DataType::Float(_), value) => value.clone().cast().map(Value::F64),
			(DataType::Text, value) => value.clone().cast().map(Value::Str),

			(DataType::Time, Value::Str(value)) => {
				let regex =
					Regex::new(r"^(\d|[0-1]\d|2[0-3]):([0-5]\d)(:([0-5]\d))? ?([AaPp][Mm])?$");
				if let Ok(regex) = regex {
					if let Some(captures) = regex.captures(value) {
						let modifier: bool = captures
							.iter()
							.last()
							.map(|capture| {
								capture
									.map(|capture| {
										Regex::new(r"^[Pp][Mm]$")
											.ok()
											.map(|regex| regex.is_match(capture.into()))
									})
									.flatten()
							})
							.flatten()
							.unwrap_or(false);
						let mut items: Vec<i64> = captures
							.iter()
							.skip(1)
							.filter_map(|capture| {
								capture
									.map(|capture| {
										let capture: &str = capture.into();
										capture.parse::<i64>().ok()
									})
									.flatten()
							})
							.collect();
						items.resize(3, 0);
						let seconds = items.iter().fold(0, |acc, item| (acc * 60) + item)
							+ if modifier { 12 * 60 * 60 } else { 0 };
						Ok(Value::I64(seconds))
					} else {
						Err(())
					}
				} else {
					Err(())
				}
				.map_err(|_| ValueError::ImpossibleCast.into())
			}

			_ => Err(ValueError::UnimplementedCast.into()),
		}
	}

	pub fn is_some(&self) -> bool {
		use Value::*;

		!matches!(self, Null)
	}

	pub fn type_max(&self) -> Self {
		match self {
			Value::Bool(_) => Value::Bool(true),
			Value::I64(_) => Value::I64(i64::MAX),
			Value::F64(_) => Value::F64(f64::MAX),
			_ => unimplemented!(), // TODO: Handle better & expand
		}
	}
	pub fn type_min(&self) -> Self {
		match self {
			Value::Bool(_) => Value::Bool(false),
			Value::I64(_) => Value::I64(i64::MIN),
			Value::F64(_) => Value::F64(f64::MIN),
			_ => unimplemented!(), // TODO: Handle better & expand
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
