use {
	crate::{Cast, Result, Value},
	std::ops::*,
	enum_dispatch::enum_dispatch,
};

#[enum_dispatch(Value)]
pub trait BinaryOperations:
	Sized + Add + Sub + Mul + Div + Rem + Eq + Ord + BitAnd + BitOr + BitXor
{
}

pub enum BinaryOperation {
	Add,
	Sub,
	Mul,
	Div,
	Rem,
	Eq,
	Lt,
	LtEq,
	Gt,
	GtEq,
	And,
	Or,
	Xor,
}

impl BinaryOperation {
	pub fn exec(&self, left: Value, right: Value) -> Result<Value> {
		use BinaryOperation::*;
		match self {
			Add => left.add(right),
			Sub => left.sub(right),
			Mul => left.mul(right),
			Div => left.div(right),
			Rem => left.rem(right),
			Eq => left == right,
			Lt => left < right,
			LtEq => left <= right,
			Gt => left > right,
			GtEq => left >= right,
			And => left & right,
			Or => left | right,
			Xor => left ^ right,
		}
	}
}

impl Value {
	pub fn string_concat(self, other: Self) -> Result<Self> {
		Ok(format!("{}{}", self.cast()?, other.cast()?).into())
	}
}
