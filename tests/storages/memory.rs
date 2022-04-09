pub fn memory(_name: &str) -> multisql::Glue {
	use multisql::*;

	let storage = Connection::Memory.try_into().expect("Create Storage");

	Glue::new(String::from("main"), storage)
}

crate::util_macros::run!(memory, functionality);
crate::util_macros::run!(memory, original);
