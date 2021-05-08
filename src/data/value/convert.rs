use {
	super::{Value, ValueError},
	crate::result::Result,
	chrono::NaiveDateTime,
};

// TODO: No clone versions

pub trait Convert<Core> {
	fn convert(self) -> Result<Core>;
}

pub trait ConvertFrom<Value>: Sized {
	fn convert_from(value: Value) -> Result<Self>;
}
impl<Core, Value> ConvertFrom<Value> for Core
where
	Value: Convert<Core> + Clone,
{
	fn convert_from(value: Value) -> Result<Core> {
		value.convert()
	}
}

impl Convert<bool> for Value {
	fn convert(self) -> Result<bool> {
		Ok(match self {
			Value::Bool(inner) => inner,
			other => return Err(ValueError::CannotConvert(other, "BOOLEAN").into()),
		})
	}
}

impl Convert<i64> for Value {
	fn convert(self) -> Result<i64> {
		Ok(match self {
			Value::I64(inner) => inner,
			other => return Err(ValueError::CannotConvert(other, "INTEGER").into()),
		})
	}
}

impl Convert<f64> for Value {
	fn convert(self) -> Result<f64> {
		Ok(match self {
			Value::F64(inner) => inner,
			#[cfg(feature = "implicit_float_conversion")]
			Value::I64(inner) => inner as f64,
			other => return Err(ValueError::CannotConvert(other, "FLOAT").into()),
		})
	}
}

impl Convert<String> for Value {
	fn convert(self) -> Result<String> {
		Ok(match self {
			Value::Str(inner) => inner,
			other => return Err(ValueError::CannotConvert(other, "TEXT").into()),
		})
	}
}

impl Convert<NaiveDateTime> for Value {
	fn convert(self) -> Result<NaiveDateTime> {
		let secs = self.convert()?;
		Ok(NaiveDateTime::from_timestamp(secs, 0))
	}
}
