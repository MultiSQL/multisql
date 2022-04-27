use {
	crate::{Result, Value},
	std::ops::*,
};

pub trait UnaryOperations: Not + Neg {}

impl Value {
	pub fn is_null(self) -> Result<Self> {
		Ok(Value::Bool(matches!(self, Value::Null(_))))
	}
}
