use {
	crate::{AlterTable, Column, MemoryStorage, Result, Schema, StorageError},
	async_trait::async_trait,
};

#[async_trait(?Send)]
impl AlterTable for MemoryStorage {}
