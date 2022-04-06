#[allow(unused_must_use)]
pub fn sheet_storage(name: &str) -> multisql::Glue {
	use {fstrings::*, multisql::*};

	let path = f!("data/sheet_{name}.xlsx");

	match std::fs::remove_file(&path) {
		Ok(()) => (),
		Err(_) => {}
	}

	std::fs::create_dir("data");

	let storage = Storage::try_from(Connection::Sheet(path)).expect("Create Storage");

	Glue::new(String::from("main"), storage)
}

crate::util_macros::run!(sheet_storage, functionality::statement::create::table);
crate::util_macros::run!(sheet_storage, functionality::statement::simple_insert);
crate::util_macros::run!(sheet_storage, functionality::statement::data_query);
//crate::util_macros::run!(sheet_storage, functionality::validation);
crate::util_macros::run!(sheet_storage, functionality::query);
crate::util_macros::run!(sheet_storage, functionality::column_options);
