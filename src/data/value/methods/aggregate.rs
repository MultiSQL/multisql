use {
	crate::{Convert, Result, Value},
	std::cmp::Ordering,
};

// This does not intentionally take into account anything that could variably change data types
// (example: IIF(column = 1, CAST(other AS INTEGER), CAST(other AS TEXT)))
// COUNT is indifferent to types,
// MIN and MAX will just give whatever the MIN/MAX of the first type (partial_cmp would evaulate to None which gives accumulator)
// SUM will, for now, use generic_add which will throw if non-artithmatic.

impl Value {
	pub fn aggregate_count(self, accumulator: Value) -> Result<Value> {
		Ok(if !matches!(self, Value::Null) {
			Value::I64(
				accumulator.convert().unwrap_or(0) /*This should only occur for first value: NULL*/ + 1,
			)
		} else {
			accumulator
		})
	}
	pub fn aggregate_min(self, accumulator: Value) -> Result<Value> {
		Ok(
			if matches!(self.partial_cmp(&accumulator), Some(Ordering::Less))
				|| matches!(accumulator, Value::Null)
			{
				self
			} else {
				accumulator
			},
		)
	}
	pub fn aggregate_max(self, accumulator: Value) -> Result<Value> {
		Ok(
			if matches!(self.partial_cmp(&accumulator), Some(Ordering::Greater))
				|| matches!(accumulator, Value::Null)
			{
				self
			} else {
				accumulator
			},
		)
	}
	pub fn aggregate_sum(self, accumulator: Value) -> Result<Value> {
		accumulator
			.if_null(Value::I64(0)) // TODO: Handle lack of implicit i64 -> f64
			.generic_add(self.clone().if_null(Value::I64(0)))
	}
}
