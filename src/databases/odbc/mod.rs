mod auto_increment;
mod base;
mod mutable;

use {
	crate::{database::*, Result},
	odbc_api::{Connection, Environment},
};

pub struct ODBCDatabase {
	environment: Environment,
	connection: Connection<'static>,
}

impl DBFull for ODBCDatabase {}

impl ODBCDatabase {
	pub fn new(connection_string: &str) -> Result<Self> {
		let environment = Environment::new()?;
		let connection = environment.connect_with_connection_string(connection_string)?;
		Ok(Self {
			environment,
			connection,
		})
	}
}
