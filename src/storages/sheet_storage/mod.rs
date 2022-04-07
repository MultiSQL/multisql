mod store;
mod store_mut;
mod auto_increment;

use {
	crate::{store::*, Result},
	async_trait::async_trait,
	serde::{Deserialize, Serialize},
	std::{fmt::Debug, path::Path},
	thiserror::Error,
	umya_spreadsheet::{new_file, reader, writer, Spreadsheet},
};

#[derive(Error, Serialize, Debug, PartialEq)]
pub enum SheetStorageError {
	#[error("FSError")]
	FSError,
	#[error("failed to parse column information")]
	FailedColumnParse,
	#[error("failed to create sheet")]
	FailedToCreateSheet,
	#[error("failed to get sheet")]
	FailedToGetSheet,
}

pub struct SheetStorage {
	book: Spreadsheet,
	path: String,
}

impl AlterTable for SheetStorage {}
impl FullStorage for SheetStorage {}

impl SheetStorage {
	pub fn new(path: &str) -> Result<Self> {
		let book = reader::xlsx::lazy_read(Path::new(path)).unwrap_or_else(|_| new_file());
		let path = path.to_string();
		Ok(Self { book, path })
	}
	pub(crate) fn save(&self) -> Result<()> {
		writer::xlsx::write(&self.book, Path::new(&self.path))
			.map_err(|_| SheetStorageError::FSError.into())
	}
}
