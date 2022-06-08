use {
	super::Glue,
	crate::{
		recipe::{Recipe, RecipeUtilities},
		Payload, Result, Value,
	},
	futures::executor::block_on,
};

#[macro_export]
macro_rules! INSERT {
	{$glue:expr, INTO $database:ident.$table:ident ($($column:ident),+) VALUES $(($($value:expr),+)),+} => {
		$glue.insert(Some(stringify!($database)), stringify!($table), &[$(stringify!($column)),+], vec![$(vec![$($value.into()),+]),+])
	};
	{$glue:expr, INTO $table:ident ($($column:ident),+) VALUES $(($($value:expr),+)),+} => {
		$glue.insert(None, stringify!($table), &[$(stringify!($column)),+], vec![$(vec![$($value.into()),+]),+])
	};
}

/// ## Insert (`INSERT`)
impl Glue {
	pub fn insert(
		&mut self,
		database: Option<&str>,
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
			&database.map(|db| db.to_string()),
			table,
			columns,
			values,
			None,
			false,
		))
	}
}

#[test]
fn test() {
	use crate::{Connection, Glue};
	let db = Connection::Memory.try_into().unwrap();
	let mut glue = Glue::new(String::from("test"), db);
	glue.execute("CREATE TABLE basic (a INT)").unwrap();
	glue.insert(None, "basic", &["a"], vec![vec![2.into()]])
		.unwrap();
	INSERT! {glue, INTO basic (a) VALUES (2)}.unwrap();
}
