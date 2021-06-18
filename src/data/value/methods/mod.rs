mod aggregate;
mod binary;
mod function;
mod timestamp;
mod unary;
mod utility;

use {
	crate::{ConvertFrom, Value},
	std::convert::Into,
};

pub trait ValueCore: Into<Value> + ConvertFrom<Value> {}
impl ValueCore for bool {}
impl ValueCore for i64 {}
impl ValueCore for f64 {}
impl ValueCore for String {}
