#[allow(unused_must_use)]
pub fn sheet_database(name: &str) -> multisql::Glue {
	use {fstrings::*, multisql::*};

	let path = f!("data/sheet_{name}.xlsx");

	match std::fs::remove_file(&path) {
		Ok(()) => (),
		Err(_) => {}
	}

	std::fs::create_dir("data");

	let database = Database::try_from(Connection::Sheet(path)).expect("Create Database");

	Glue::new(String::from("main"), database)
}

//crate::util_macros::run!(sheet_database, api);
crate::util_macros::run!(sheet_database, original);
crate::util_macros::run!(sheet_database, functionality::statement::create::table);
crate::util_macros::run!(sheet_database, functionality::statement::simple_insert);
crate::util_macros::run!(sheet_database, functionality::statement::data_query);
crate::util_macros::run!(sheet_database, functionality::validation);
crate::util_macros::run!(sheet_database, functionality::query::join);
crate::util_macros::run!(sheet_database, functionality::query::aggregate);
crate::util_macros::run!(sheet_database, functionality::query::function);
//crate::util_macros::run!(sheet_database, functionality::api);
//crate::util_macros::run!(sheet_database, functionality::query);
//crate::util_macros::run!(sheet_database, functionality::column_options);
