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
}

fn main() {
	for database in inventory::iter::<TestDatabase> {
		println!(
			"\nTesting database:\t {}\n- - -\t- - -\t- - -\t\t- - -\t- - -\t- - -\t\t- - -\t- - -\t- - -",
			database.name
		);
		for test in inventory::iter::<Test> {
			run!(test, database.init);
		}
	}
}
