inventory::submit!(crate::TestDatabase {
	init: database,
	name: "CSV",
});
pub fn database(name: &str) -> multisql::Glue {
	use multisql::*;
	let path = format!("data/csv_{}.csv", name.replace("::", "_"));
	if std::fs::remove_file(&path).is_ok() {
		println!("Old file removed");
	}
	if std::fs::create_dir("data").is_ok() {
		println!("'Data' directory created");
	}
	let database = CSVDatabase::new(&path)
		.map(Database::new_csv)
		.expect("Create Database");

	Glue::new(String::from("main"), database)
}
