mod store;
mod store_mut;

use {
	crate::{data::Schema, store::*, FullStorage, Result, Storage, Error},
	calamine::{Reader, open_workbook},
	csv::ReaderBuilder,
	serde::{Deserialize, Serialize},
	sqlparser::ast::{ColumnDef, DataType, Ident},
	std::{
		default::Default,
		fmt::Debug,
		fs::{File, OpenOptions},
		io::BufReader,
	},
};

pub struct SheetStorage {
	path: String,
}

impl AlterTable for SheetStorage {}
impl AutoIncrement for SheetStorage {}
impl FullStorage for SheetStorage {}

impl SheetStorage {
	pub fn new(path: &str) -> Result<Self> {
		let path = path.to_string();
		Ok(Self { path })
	}
	pub fn workbook<Sheet>(&self) -> Sheet
	where Sheet: Reader<RS = BufReader<File>> + Sized {
		if let Ok(workbook) = open_workbook(&self.path) {
			workbook
		} else {
			panic!()
		}
	}
}
