use {crate::{MemoryStorage, AutoIncrement}, async_trait::async_trait};

#[async_trait(?Send)]
impl AutoIncrement for MemoryStorage {
}
