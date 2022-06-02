use {
	super::Glue,
	crate::{
		recipe::{Recipe, RecipeUtilities},
		Payload, Result, Value,
	},
	futures::executor::block_on,
};

pub trait ParameterValue {
	fn into_recipe(self) -> Recipe;
}

#[macro_export]
macro_rules! INSERT {
	{$glue:expr, INTO $database:ident.$table:ident ($($column:ident),*) VALUES $(($($value:expr),*)),*} => {
		$glue.insert(stringify!($database), stringify!($table), &[$(stringify!($column)),*], vec![$(vec![$($value.into()),*]),*]);
	}
}

/// ## Insert (`INSERT`)
impl Glue {
	pub fn insert(
		&mut self,
		database: &str,
		table: &str,
		columns: &[&str],
		recipes: Vec<Vec<Recipe>>,
	) -> Result<Payload> {
		let values = recipes
			.into_iter()
			.map(|recipes| {
				recipes
					.into_iter()
					.map(|recipe| recipe.simplify_by_basic()?.confirm())
					.collect::<Result<Vec<Value>>>()
			})
			.collect::<Result<Vec<Vec<Value>>>>()?;
		block_on(self.true_insert(
			&Some(database.to_string()),
			table,
			columns,
			values,
			None,
			false,
		))
	}
}
