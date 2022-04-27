use {
	crate::{Result, Value, Cast},
	std::ops::*,
};

pub trait BinaryOperations:
	Sized + Add + Sub + Mul + Div + Rem + Eq + Ord + BitAnd + BitOr + BitXor
{
}

impl Value {
	pub fn string_concat(self, other: Self) -> Result<Self> {
		Ok(format!("{}{}", self.cast()?, other.cast()?).into())
	}
}
