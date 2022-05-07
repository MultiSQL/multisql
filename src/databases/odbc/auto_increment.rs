use {
	crate::{AutoIncrement, ODBCDatabase},
	async_trait::async_trait,
};

#[async_trait(?Send)]
impl AutoIncrement for ODBCDatabase {}
