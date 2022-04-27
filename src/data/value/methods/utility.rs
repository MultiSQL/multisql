use {
	crate::{Convert, Result, Value, ValueError},
	std::cmp::min,
};

macro_rules! protect_null {
	($protect: expr) => {
		if matches!($protect, Value::Null(_)) {
			return Ok($protect);
		}
	};
}

impl Value {
	pub fn if_null(self, alternative: Self) -> Self {
		if !matches!(self, Value::Null(_)) {
			self
		} else {
			alternative
		}
	}
	pub fn null_if(self, evaluate: Self) -> Result<Self> {
		Ok(if self == evaluate { Value::NULL } else { self })
	}
	pub fn iif(self, case_true: Self, case_false: Self) -> Result<Self> {
		Ok(if self.convert()? {
			case_true
		} else {
			case_false
		})
	}

	pub fn to_uppercase(self) -> Result<Self> {
		protect_null!(self);
		let string: String = self.convert()?;
		Ok(string.to_uppercase().into())
	}
	pub fn to_lowercase(self) -> Result<Self> {
		protect_null!(self);
		let string: String = self.convert()?;
		Ok(string.to_lowercase().into())
	}
	pub fn left(self, length: Value) -> Result<Value> {
		protect_null!(self);
		protect_null!(length);
		let length: i64 = length.convert()?;
		if length < 0 {
			return Err(ValueError::BadInput(length.into()).into());
		}
		let length: usize = length as usize;
		let string: String = self.convert()?;

		let truncated = string
			.get(..length)
			.map(|result| result.to_string())
			.unwrap_or(string);
		Ok(Value::Str(truncated))
	}
	pub fn right(self, length: Value) -> Result<Value> {
		protect_null!(self);
		protect_null!(length);
		let length: i64 = length.convert()?;
		if length < 0 {
			return Err(ValueError::BadInput(length.into()).into());
		}
		let length: usize = length as usize;
		let string: String = self.convert()?;

		let truncated = string
			.get(string.len() - min(string.len(), length)..)
			.map(|result| result.to_string())
			.unwrap_or(string);
		Ok(Value::Str(truncated))
	}
	pub fn length(self) -> Result<Value> {
		let string: String = self.convert()?;
		Ok(Value::I64(string.len() as i64))
	}

	pub fn concat(self, strings: Vec<Value>) -> Result<Value> {
		strings
			.into_iter()
			.try_fold(self, |all, this| all.string_concat(this))
	}

	pub fn replace(self, from: Value, to: Value) -> Result<Value> {
		protect_null!(self);
		let string: String = self.convert()?;
		let from: String = from.convert()?;
		let to: String = to.convert()?;

		Ok(string.replace(&from, &to).into())
	}

	pub fn round(self, places: Value) -> Result<Value> {
		if matches!(self, Value::Null(_)) {
			return Ok(self);
		}
		let value: f64 = self.convert()?;
		let places: i64 = places.convert()?;
		let raiser: f64 = 10_u32.pow(places as u32).into();
		Ok(Value::F64((value * raiser).round() / raiser))
	}
	pub fn pow(self, power: Value) -> Result<Value> {
		let value: f64 = self.convert()?;
		let power: f64 = power.convert()?;
		Ok(Value::F64(value.powf(power)))
	}
}
