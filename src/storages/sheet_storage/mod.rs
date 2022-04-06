mod store;
mod store_mut;

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

#[async_trait(?Send)]
impl AutoIncrement for SheetStorage {
	// TODO: Move
	async fn generate_increment_values(
		&mut self,
		sheet_name: String,
		columns: Vec<(
			usize,  /*index*/
			String, /*name*/
			i64,    /*row_count*/
		) /*column*/>,
	) -> Result<
		Vec<(
			/*column*/ (usize /*index*/, String /*name*/),
			/*start_value*/ i64,
		)>,
	> {
		let sheet = self.book.get_sheet_by_name_mut(sheet_name).unwrap();
		let row_init = sheet.get_row_dimensions().len() + 1;
		Ok(columns
			.into_iter()
			.map(|(index, name, _)| ((index, name), row_init as i64))
			.collect())
	}
}
