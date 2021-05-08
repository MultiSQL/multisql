static mut CSV_NUM: u16 = 0;
macro_rules! storage {
	() => {{
		use {fstrings::*, multisql::*};

		let path = unsafe {
			CSV_NUM = CSV_NUM + 1;
			f!("data/csv_{CSV_NUM}.csv")
		};

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
	}};
}
pub fn csv_storage() -> multisql::Glue {
	storage!()
}

crate::functionality::statement::create::all!(csv_storage);
crate::functionality::statement::insert::all!(csv_storage);
crate::functionality::statement::select::all!(csv_storage);
