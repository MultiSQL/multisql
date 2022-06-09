mod auto_increment;
mod base;
mod mutable;

use {
	crate::{database::*, Result},
	serde::Serialize,
	std::{fmt::Debug, path::Path},
	thiserror::Error,
	umya_spreadsheet::{new_file_empty_worksheet, reader, writer, Spreadsheet, Worksheet},
};

#[derive(Error, Serialize, Debug, PartialEq)]
pub enum SheetDatabaseError {
	#[error("File System Error: {0}")]
	FSError(String),
	#[error("failed to parse column information")]
	FailedColumnParse,
	#[error("failed to create sheet")]
	FailedToCreateSheet,
	#[error("failed to get sheet")]
	FailedToGetSheet,
}

pub struct SheetDatabase {
	book: Spreadsheet,
	path: String,
}

impl DBFull for SheetDatabase {}

impl SheetDatabase {
	pub fn new(path: &str) -> Result<Self> {
		let book =
			reader::xlsx::read(Path::new(path)).unwrap_or_else(|_| new_file_empty_worksheet());
		let path = path.to_string();
		Ok(Self { book, path })
	}
	pub(crate) fn save(&self) -> Result<()> {
		writer::xlsx::write(&self.book, Path::new(&self.path))
			.map_err(|e| format!("{:?}", e))
			.map_err(SheetDatabaseError::FSError)
			.map_err(crate::Error::SheetDatabase)
	}

	pub(crate) fn get_sheet_mut(&mut self, sheet_name: &str) -> Result<&mut Worksheet> {
		self.book
			.get_sheet_by_name_mut(sheet_name)
			.map_err(|_| SheetDatabaseError::FailedToGetSheet.into())
	}
}
