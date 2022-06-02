use {
	super::Glue,
	crate::{recipe::Recipe, Cast, ExecuteError, Payload, Result, Value},
	serde_json::{json, value::Value as JSONValue},
};

trait ParameterValue {
	fn into_recipe(self) -> Recipe;
}
impl ParameterValue for Value {
	fn into_recipe(self) -> Recipe {
		unimplemented!()
	}
}

/// ## Insert (`INSERT`)
impl Glue {
	pub fn insert(
		&mut self,
		table: &str,
		columns: &[&str],
		values: Vec<Recipe>,
	) -> Result<Payload> {
		unimplemented!()
	}
}
