mod auto_increment;
mod base;
mod mutable;

use {
	crate::{database::*, Result},
	odbc_api::Environment,
};

pub struct ODBCDatabase {
	environment: Environment,
	connection_string: String,
}

impl DBFull for ODBCDatabase {}

impl ODBCDatabase {
	pub fn new(connection_string: &str) -> Result<Self> {
		let environment = Environment::new()?;
		environment.connect_with_connection_string(connection_string)?; // Fail Fast
		let connection_string = connection_string.to_string();
		Ok(Self {
			environment,
			connection_string,
		})
	}
}

#[test]
fn temp_odbc_test() {
	use crate::{Connection, Glue};
	let connection = Connection::ODBC(String::from("Driver={SQL Server}; Server=CPServer18; Database=CostProBI_Common; Uid=kyran; Pwd=KyGost77; Trusted_Connection=yes"));
	let database = connection.try_into().unwrap();
	let glue = Glue::new(String::from("main"), database);
	panic!();
}
