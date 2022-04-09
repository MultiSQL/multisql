use {
	crate::{AutoIncrement, MemoryStorage},
	async_trait::async_trait,
};

#[async_trait(?Send)]
impl AutoIncrement for MemoryStorage {}
