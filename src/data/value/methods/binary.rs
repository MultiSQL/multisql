use {
	crate::{Cast, Result, Value, Valued},
	std::ops::*,
};

pub trait BinaryOperations<O: Valued>:
	Sized + ValuedBinaryOperations<O> + BooleanBinaryOperations
{
}

pub trait ValuedBinaryOperations<O: Valued>:
	Sized
	+ Add<Output = O>
	+ Sub<Output = O>
	+ Mul<Output = O>
	+ Div<Output = O>
	+ Rem<Output = O>
	+ BitAnd<Output = O>
	+ BitOr<Output = O>
	+ BitXor<Output = O>
{
}

pub trait BooleanBinaryOperations: Sized + Eq + Ord {}

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
	pub fn exec<I: BinaryOperations<O>, O: Valued>(&self, left: I, right: I) -> Result<Value> {
		use BinaryOperation::*;
		Ok(match self {
			Add => left.add(right).into(),
			Sub => left.sub(right).into(),
			Mul => left.mul(right).into(),
			Div => left.div(right).into(),
			Rem => left.rem(right).into(),
			Eq => (left == right).into(),
			Lt => (left < right).into(),
			LtEq => (left <= right).into(),
			Gt => (left > right).into(),
			GtEq => (left >= right).into(),
			And => (left & right).into(),
			Or => (left | right).into(),
			Xor => (left ^ right).into(),
		})
	}
}

/*impl Value {
	pub fn string_concat(self, other: Self) -> Result<Self> {
		Ok(format!("{}{}", self.cast()?, other.cast()?).into())
	}
}*/
