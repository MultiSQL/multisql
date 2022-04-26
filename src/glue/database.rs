use {
	crate::{Context, Database, DatabaseInner, Glue, InterfaceError, Result},
	std::sync::MutexGuard,
};

impl Glue {
	// TODO: None ref should give a primary
	pub fn get_database(&self, db_ref: &Option<String>) -> Result<MutexGuard<Box<DatabaseInner>>> {
		self.databases
			.get(db_ref.as_ref().unwrap_or(&self.primary))
			.ok_or(InterfaceError::DatabaseNotFound.into())
			.map(|db| db.get())
	}
	pub fn get_mut_database(&mut self, db_ref: &Option<String>) -> Result<&mut Box<DatabaseInner>> {
		self.databases
			.get_mut(db_ref.as_ref().unwrap_or(&self.primary))
			.ok_or(InterfaceError::DatabaseNotFound.into())
			.map(Database::get_mut)
	}
	pub fn get_context(&self) -> Result<MutexGuard<Context>> {
		self.context
			.lock()
			.map_err(|_| InterfaceError::ContextUnavailable.into())
	}
	pub fn get_mut_context(&mut self) -> Result<&mut Context> {
		self.context
			.get_mut()
			.map_err(|_| InterfaceError::ContextUnavailable.into())
	}
	pub fn get_database_list(&self) -> Vec<&String> {
		self.databases.keys().collect()
	}
}
