pub fn sled_storage(name: &str) -> multisql::Glue {
	use {fstrings::*, multisql::*};

	println_f!("{name}");
	let path = f!("data/sled_{name}");

	match std::fs::remove_dir_all(&path) {
		Ok(()) => (),
		Err(e) => {
			println!("fs::remove_file {:?}", e);
		}
	}

	let storage = SledStorage::new(&path)
		.map(Storage::new_sled)
		.expect("Create Storage");

	Glue::new(String::from("main"), storage)
}

crate::util_macros::run!(sled_storage, functionality);
crate::util_macros::run!(sled_storage, original);
