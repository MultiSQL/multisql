inventory::submit!(crate::TestDatabase {
	init: database,
	name: "Sheet",
	exceptions: &[
		"alter",
		"column_option::auto_increment",
		"index",
		"base::generic_complex"
	]
});
pub fn database(name: &str) -> multisql::Glue {
	use multisql::*;
	let path = format!("data/sheet_{}.xlsx", name.replace("::", "_"));
	if std::fs::remove_file(&path).is_ok() {
		//println!("Old file removed");
	}
	if std::fs::create_dir("data").is_ok() {
		//println!("'Data' directory created");
	}
	let database = Database::try_from(Connection::Sheet(path)).expect("Create Database");
	Glue::new(String::from("main"), database)
}
