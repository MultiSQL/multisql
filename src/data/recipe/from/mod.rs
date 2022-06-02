mod method;
pub use method::*;

use {super::Recipe, crate::Value};

impl<T: Into<Value>> From<T> for Recipe {
	fn from(value: T) -> Recipe {
		let value: Value = value.into();
		value.into()
	}
}
