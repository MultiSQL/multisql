use {
	crate::{Context, Glue, InterfaceError, Result, Storage, StorageInner},
	std::sync::MutexGuard,
};

impl Glue {
	// TODO: None ref should give a primary
	pub fn get_database(&self, db_ref: &Option<String>) -> Result<MutexGuard<Box<StorageInner>>> {
		db_ref
			.as_ref()
			.and_then(|db_ref| self.databases.get(db_ref))
			.ok_or(InterfaceError::DatabaseNotFound.into())
			.map(|db| db.get())
	}
	pub fn get_mut_database(&mut self, db_ref: &Option<String>) -> Result<&mut Box<StorageInner>> {
		db_ref
			.as_ref()
			.and_then(|db_ref| self.databases.get_mut(db_ref))
			.ok_or(InterfaceError::DatabaseNotFound.into())
			.map(Storage::get_mut)
	}
	pub fn get_context(&self) -> Result<&Context> {
		self.context
			.as_ref()
			.ok_or(InterfaceError::ContextUnavailable.into())
	}
	pub fn get_mut_context(&self) -> Result<&mut Context> {
		Err(InterfaceError::ContextUnavailable.into())
	}
	pub fn get_database_list(&self) -> Vec<&String> {
		self.databases.keys().collect()
	}
	/*pub fn database_iter(&self) -> Result<Box<dyn Iterator<Item = (&String, &StorageInner)>>> {
		Ok(Box::new(self.databases.iter().map(|(db_ref, db)| (db_ref, &*db.take()))))
	}*/
}
