use {
	crate::{store::*, Result, SheetStorage, SheetStorageError},
	async_trait::async_trait,
	serde::{Deserialize, Serialize},
	std::{fmt::Debug, path::Path},
	thiserror::Error,
	umya_spreadsheet::{new_file, reader, writer, Spreadsheet},
};


#[async_trait(?Send)]
impl AutoIncrement for SheetStorage {
	// TODO: Move
	async fn generate_increment_values(
		&mut self,
		sheet_name: String,
		columns: Vec<(usize, String, i64)>,
	) -> Result<Vec<((usize, String), i64)>> {
		let sheet = self
			.book
			.get_sheet_by_name_mut(sheet_name)
			.map_err(|_| SheetStorageError::FailedToGetSheet)?;
		let row_init = sheet.get_row_dimensions().len();
		Ok(columns
			.into_iter()
			.map(|(index, name, _)| ((index, name), row_init as i64))
			.collect())
	}
}
