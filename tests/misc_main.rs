mod misc;
mod util;

use {
	multisql::{Connection, Glue},
	util::*,
};

inventory::collect!(Test);

fn main() {
	announce_test_suite!();
	for test in inventory::iter::<Test> {
		run!(test, init_storage);
	}
}

fn init_storage(_test: &str) -> Glue {
	let db = Connection::Memory.try_into().unwrap();
	Glue::new("main".into(), db)
}
