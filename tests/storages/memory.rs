pub fn memory(_name: &str) -> multisql::Glue {
	use multisql::*;

	let database = Connection::Memory.try_into().expect("Create Database");

	Glue::new(String::from("main"), database)
}

/*crate::util_macros::run!(memory, functionality);
crate::util_macros::run!(memory, original);*/
