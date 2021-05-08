static mut SLED_NUM: u16 = 0;
macro_rules! storage {
	() => {{
		use {fstrings::*, multisql::*};

		let path = unsafe {
			SLED_NUM = SLED_NUM + 1;
			f!("data/sled_{SLED_NUM}")
		};

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
	}};
}
pub fn sled_storage() -> multisql::Glue {
	storage!()
}

crate::functionality::all!(sled_storage);
crate::original::all!(sled_storage);
