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
			if !database.exceptions.iter().any(|exception| {
				test.name
					.starts_with(&format!("databases::ability::{}", exception))
			}) {
				run!(test, database.init);
			}
		}
	}
}
