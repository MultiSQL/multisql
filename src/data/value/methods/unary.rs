use {
	crate::{Result, Value},
	enum_dispatch::enum_dispatch,
	std::ops::*,
};

pub trait UnaryOperations: Not + Neg {}

pub enum UnaryOperation {}
impl Value {
	pub fn is_null(self) -> Result<Self> {
		Ok(Value::Bool(matches!(self, Value::Null(_))))
	}
}
