mod ability;
mod databases;
mod util;

use {
	multisql::Glue,
	util::{run, Test},
};

inventory::collect!(Test);
inventory::collect!(TestDatabase);
struct TestDatabase {
	init: fn(&str) -> Glue,
	name: &'static str,
	exceptions: &'static [&'static str],
}

fn main() {
	for database in inventory::iter::<TestDatabase> {
		println!(
			"- - -\t- - -\t- - -\t- - -\t- - -\t- - -\t- - -\t- - -\t- - -
			\nTesting database:\t {}
			\n- - -\t- - -\t- - -\t- - -\t- - -\t- - -\t- - -\t- - -\t- - -",
			database.name
		);
		for test in inventory::iter::<Test> {
			let name = test.name.strip_prefix(concat!(module_path!(), "::", "ability::")).unwrap();
			if !database
				.exceptions
				.iter()
				.any(|exception| name.starts_with(exception))
			{
				run!(test, database.init);
			}
		}
	}
}
