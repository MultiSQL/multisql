use {
	crate::{Result, Value},
	std::cmp::Ordering,
};

// This does not intentionally take into account anything that could variably change data types
// (example: IIF(column = 1, CAST(other AS INTEGER), CAST(other AS TEXT)))
// COUNT is indifferent to types,
// MIN and MAX will just give whatever the MIN/MAX of the first type (partial_cmp would evaulate to None which gives accumulator)
// SUM will, for now, use generic_add which will throw if non-artithmatic.

impl Value {
	pub fn aggregate_count(self, other: Value) -> Result<Value> {
		Ok(Value::I64(match (self, other) {
			(Value::Null, Value::Null) => 0,
			(Value::I64(self_val), Value::I64(other_val)) => self_val + other_val, // WILL CAUSE ISSUES IF VALUE IS NULL! TODO: Do this another way
			(Value::I64(val), Value::Null) | (Value::Null, Value::I64(val)) => val,
			(Value::I64(val), _) | (_, Value::I64(val)) => val + 1,
			(_, _) => 2,
		}))
	}
	pub fn aggregate_min(self, other: Value) -> Result<Value> {
		Ok(
			if matches!(self.partial_cmp(&other), Some(Ordering::Less))
				|| matches!(other, Value::Null)
			{
				self
			} else {
				other
			},
		)
	}
	pub fn aggregate_max(self, other: Value) -> Result<Value> {
		Ok(
			if matches!(self.partial_cmp(&other), Some(Ordering::Greater))
				|| matches!(other, Value::Null)
			{
				self
			} else {
				other
			},
		)
	}
	pub fn aggregate_sum(self, other: Value) -> Result<Value> {
		other
			.if_null(Value::I64(0)) // TODO: Handle lack of implicit i64 -> f64
			.generic_add(self.if_null(Value::I64(0)))
	}
}
