inventory::submit!(crate::TestDatabase {
	init: database,
	name: "Sled",
	exceptions: &[]
});
pub fn database(name: &str) -> multisql::Glue {
	use multisql::*;
	let path = format!("data/sled_{}", name.replace("::", "_"));
	if std::fs::remove_dir_all(&path).is_ok() {
		//println!("Old directory removed");
	}
	if std::fs::create_dir("data").is_ok() {
		//println!("'Data' directory created");
	}
	let database = SledDatabase::new(&path)
		.map(Database::new_sled)
		.expect("Create Database");

	Glue::new(String::from("main"), database)
}
