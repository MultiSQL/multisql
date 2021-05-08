#![cfg(feature = "csv-storage")]

use std::{cell::RefCell, rc::Rc};

use multisql::{tests::*, CSVStorage, Storage};

struct CSVTester {
	storage: Rc<RefCell<Option<Storage>>>,
}

impl Tester for CSVTester {
	fn new(namespace: &str) -> Self {
		let path = format!("data/{}.csv", namespace);

		match std::fs::remove_file(&path) {
			Ok(()) => (),
			Err(e) => {
				println!("fs::remove_file {:?}", e);
			}
		}

		let storage = CSVStorage::new(&path)
			.map(Storage::new_csv)
			.map(Some)
			.map(RefCell::new)
			.map(Rc::new)
			.expect("New CSV Tester");

		CSVTester { storage }
	}

	fn get_cell(&mut self) -> Rc<RefCell<Option<Storage>>> {
		Rc::clone(&self.storage)
	}
}

//multisql::generate_tests!(tokio::test, CSVTester);
