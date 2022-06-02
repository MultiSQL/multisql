mod method;
pub use method::*;

use {
	super::{Ingredient, Recipe},
	crate::Value,
};

impl<T: Into<Value>> From<T> for Recipe {
	fn from(value: T) -> Recipe {
		Recipe::Ingredient(Ingredient::Value(value.into()))
	}
}
