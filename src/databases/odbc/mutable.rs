use {
	crate::{
		DBMut, ODBCDatabase,
	},
	async_trait::async_trait,
};

#[async_trait(?Send)]
impl DBMut for ODBCDatabase {

}
