mod alter_table;
mod auto_increment;
mod store;
mod store_mut;

use {
	crate::{store::*, Result},
	serde::Serialize,
	std::{fmt::Debug, path::Path},
	thiserror::Error,
	umya_spreadsheet::{new_file, reader, writer, Spreadsheet, Worksheet},
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

	pub(crate) fn get_sheet_mut(&mut self, sheet_name: &str) -> Result<&mut Worksheet> {
		self.book
			.get_sheet_by_name_mut(sheet_name)
			.map_err(|_| SheetStorageError::FailedToGetSheet.into())
	}
}
