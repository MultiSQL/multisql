use {
	super::Glue,
	crate::{recipe::Recipe, Cast, ExecuteError, Payload, Result, Value},
	serde_json::{json, value::Value as JSONValue},
};

pub trait ParameterValue {
	fn into_recipe(self) -> Recipe;
}
impl<T: Into<Recipe>> ParameterValue for T {
	fn into_recipe(self) -> Recipe {
		self.into()
	}
}

#[macro_export]
macro_rules! INSERT {
	{$glue:expr, INTO $table:ident ($($column:ident),*) VALUES $(($($value:expr),*)),*} => {
		$glue.insert(stringify!($table), &[$(stringify!($column)),*], vec![$(vec![$(multisql::ParameterValue::into_recipe($value)),*]),*]);
	}
}

/// ## Insert (`INSERT`)
impl Glue {
	pub fn insert(
		&mut self,
		table: &str,
		columns: &[&str],
		values: Vec<Vec<Recipe>>,
	) -> Result<Payload> {
		unimplemented!()
	}
}
