mod functionality;
mod util;

use {
	multisql::{Connection, Glue},
	util::{run, Test},
};

inventory::collect!(Test);

fn main() {
	for test in inventory::iter::<Test> {
		run!(test, init_storage);
	}
}

fn init_storage(_test: &str) -> Glue {
	let db = Connection::Memory.try_into().unwrap();
	Glue::new("main".into(), db)
}
