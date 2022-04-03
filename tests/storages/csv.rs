pub fn csv_storage(name: &str) -> multisql::Glue {
	use {fstrings::*, multisql::*};

	let path = f!("data/csv_{name}.csv");

	match std::fs::remove_file(&path) {
		Ok(()) => (),
		Err(e) => {
			println!("fs::remove_file {:?}", e);
		}
	}

	let storage = CSVStorage::new(&path)
		.map(Storage::new_csv)
		.expect("Create Storage");

	Glue::new(String::from("main"), storage)
}

crate::util_macros::run!(csv_storage, functionality::statement::create);
crate::util_macros::run!(csv_storage, functionality::statement::insert);
crate::util_macros::run!(csv_storage, functionality::statement::data_query);
