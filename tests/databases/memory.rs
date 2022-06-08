inventory::submit!(crate::TestDatabase {
	init: database,
	name: "Memory"
});
pub fn database(_name: &str) -> multisql::Glue {
	use multisql::*;

	let database = Connection::Memory.try_into().expect("Create Database");

	Glue::new(String::from("main"), database)
}
