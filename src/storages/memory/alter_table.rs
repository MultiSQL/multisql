use {
	crate::{Column, Result, Schema, MemoryStorage, StorageError, AlterTable},
	async_trait::async_trait,
};

#[async_trait(?Send)]
impl AlterTable for MemoryStorage {
}
