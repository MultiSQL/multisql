mod auto_increment;
mod base;
mod discern;
mod mutable;
mod record;
mod utils;

pub use {discern::*, record::*};

use {
	crate::{data::Schema, DBFull, Database, Result, WIPError},
	serde::{Deserialize, Serialize},
	std::{
		default::Default,
		fmt::Debug,
		fs::{OpenOptions},
	},
	thiserror::Error,
};

#[derive(Error, Serialize, Debug, PartialEq)]
pub enum CSVDatabaseError {
	#[error("CSV storages only support one table at a time")]
	OnlyOneTableAllowed,

	#[error("Failed to open CSV because of a error with header: {0}")]
	HeaderError(String),
}

pub struct CSVDatabase {
	schema: Option<Schema>,
	path: String,
	pub csv_settings: CSVSettings,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct CSVSettings {
	pub delimiter: u8,
	pub quoting: bool,
	pub has_header: Option<bool>,
	pub sample_rows: usize,
}
impl Default for CSVSettings {
	fn default() -> Self {
		Self {
			delimiter: b',',
			quoting: true,
			has_header: None,
			sample_rows: 100,
		}
	}
}

impl DBFull for CSVDatabase {}

impl Database {
	pub fn new_csv(storage: CSVDatabase) -> Self {
		Self::new(Box::new(storage))
	}
}
impl CSVDatabase {
	pub fn new(path: &str) -> Result<Self> {
		Self::new_with_settings(path, CSVSettings::default())
	}
	pub fn new_with_settings(path: &str, mut csv_settings: CSVSettings) -> Result<Self> {
		let file = OpenOptions::new()
			.read(true)
			.write(true)
			.create(true)
			.open(path)
			.map_err(|error| WIPError::Debug(format!("{:?}", error)))?;

		let schema = csv_settings.discern_schema(file)?;
		Ok(Self {
			schema,
			path: path.to_string(),
			csv_settings,
		})
	}
}
