use {
	super::ValueCore,
	crate::{Convert, ConvertFrom, Result, Value, ValueError},
	std::ops::Neg,
};

macro_rules! generic {
	($name: ident, $generic_name: ident) => {
		pub fn $generic_name(self) -> Result<Self> {
			if !i64::convert_from(self.clone()).is_err() {
				// TODO: Improve
				self.$name::<i64>()
			} else if !f64::convert_from(self.clone()).is_err() {
				self.$name::<f64>()
			} else {
				Err(ValueError::OnlySupportsNumeric(self, stringify!($name)).into())
			}
		}
	};
}

impl Value {
	pub fn unary_plus<Core>(self) -> Result<Self>
	where
		Core: ValueCore + Clone,
	{
		let core = Core::convert_from(self)?;
		let result = core.clone();
		Ok(result.into())
	}
	pub fn unary_minus<Core>(self) -> Result<Self>
	where
		Core: ValueCore + Neg<Output = Core>,
	{
		let core = Core::convert_from(self)?;
		let result = -core;
		Ok(result.into())
	}

	generic!(unary_plus, generic_unary_plus);
	generic!(unary_minus, generic_unary_minus);

	pub fn not(self) -> Result<Self> {
		let boolean: bool = self.convert()?;
		let result = !boolean;
		Ok(result.into())
	}
	pub fn is_null(self) -> Result<Self> {
		Ok(Value::Bool(matches!(self, Value::Null)))
	}
}
