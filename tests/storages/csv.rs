#[allow(unused_must_use)]
pub fn csv_database(name: &str) -> multisql::Glue {
	use {fstrings::*, multisql::*};

	let path = f!("data/csv_{name}.csv");

	match std::fs::remove_file(&path) {
		Ok(()) => (),
		Err(e) => {
			println!("fs::remove_file {:?}", e);
		}
	}

	std::fs::create_dir("data");

	let database = CSVDatabase::new(&path)
		.map(Database::new_csv)
		.expect("Create Database");

	Glue::new(String::from("main"), database)
}

crate::util_macros::run!(csv_database, functionality::statement::create::table);
crate::util_macros::run!(csv_database, functionality::statement::simple_insert);
crate::util_macros::run!(csv_database, functionality::statement::data_query);
