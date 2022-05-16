mod auto_increment;
mod base;
mod column_set;
mod mutable;

pub(crate) use column_set::ColumnSet;
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
