pub fn sled_database(name: &str) -> multisql::Glue {
	use {fstrings::*, multisql::*};

	let path = f!("data/sled_{name}");

	match std::fs::remove_dir_all(&path) {
		Ok(()) => (),
		Err(e) => {
			println!("fs::remove_file {:?}", e);
		}
	}

	let database = SledDatabase::new(&path)
		.map(Database::new_sled)
		.expect("Create Database");

	Glue::new(String::from("main"), database)
}

crate::util_macros::run!(sled_database, functionality);
crate::util_macros::run!(sled_database, original);
