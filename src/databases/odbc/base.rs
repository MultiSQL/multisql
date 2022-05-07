use {
	crate::{DBBase, ODBCDatabase, },
	async_trait::async_trait,
};

#[async_trait(?Send)]
impl DBBase for ODBCDatabase {}
