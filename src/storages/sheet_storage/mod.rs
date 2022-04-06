mod store;
mod store_mut;

use {
	crate::{store::*, Result},
	std::{path::Path,
		fmt::Debug,
	},
	umya_spreadsheet::{new_file, reader, writer, Spreadsheet},
	serde::{Deserialize, Serialize},
	thiserror::Error,
};

#[derive(Error, Serialize, Debug, PartialEq)]
pub enum SheetStorageError {
	#[error("FSError")]
	FSError,
}

pub struct SheetStorage {
	book: Spreadsheet,
	path: String,
}

impl AlterTable for SheetStorage {}
impl AutoIncrement for SheetStorage {}
impl FullStorage for SheetStorage {}

impl SheetStorage {
	pub fn new(path: &str) -> Result<Self> {
		let book = reader::xlsx::lazy_read(Path::new(path)).unwrap_or_else(|_| new_file());
		let path = path.to_string();
		Ok(Self { book, path })
	}
	pub(crate) fn save(&self) -> Result<()> {
		writer::xlsx::write(&self.book, Path::new(&self.path)).map_err(|_|SheetStorageError::FSError.into())
	}
}
