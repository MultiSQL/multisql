mod auto_increment;
mod store;
mod store_mut;
mod utils;
/*mod alter_table;
mod error;
#[cfg(not(feature = "alter-table"))]
impl crate::AlterTable for CSVStorage {}
#[cfg(not(feature = "auto-increment"))]
impl crate::AutoIncrement for CSVStorage {}*/

use {
	crate::{data::Schema, store::*, FullStorage, Result, Storage, WIPError},
	csv::ReaderBuilder,
	serde::Serialize,
	sqlparser::ast::{ColumnDef, DataType, Ident},
	std::{
		default::Default,
		fmt::Debug,
		fs::{File, OpenOptions},
	},
	thiserror::Error,
};

#[derive(Error, Serialize, Debug, PartialEq)]
pub enum CSVStorageError {
	#[error("CSV storages only support one table at a time")]
	OnlyOneTableAllowed,
}

pub struct CSVStorage {
	schema: Option<Schema>,
	path: String,
	pub csv_settings: CSVSettings,
}
#[derive(Clone)]
pub struct CSVSettings {
	pub delimiter: u8,
}
impl Default for CSVSettings {
	fn default() -> Self {
		Self { delimiter: b',' }
	}
}

impl AlterTable for CSVStorage {}
impl FullStorage for CSVStorage {}

impl Storage {
	pub fn new_csv(storage: CSVStorage) -> Self {
		Self::new(Box::new(storage))
	}
}
impl CSVStorage {
	pub fn new(path: &str) -> Result<Self> {
		Self::new_with_settings(path, CSVSettings::default())
	}
	pub fn new_with_settings(path: &str, csv_settings: CSVSettings) -> Result<Self> {
		let file = OpenOptions::new()
			.read(true)
			.write(true)
			.create(true)
			.open(path)
			.map_err(|error| WIPError::Debug(format!("{:?}", error)))?;

		let schema = discern_schema(file, &csv_settings)?;
		Ok(Self {
			schema,
			path: path.to_string(),
			csv_settings,
		})
	}
}

fn discern_schema(file: File, csv_settings: &CSVSettings) -> Result<Option<Schema>> {
	let mut reader = ReaderBuilder::new()
		.delimiter(csv_settings.delimiter)
		.from_reader(file);
	let headers = reader
		.headers()
		.map_err(|error| WIPError::Debug(format!("{:?}", error)))?;
	let column_defs = headers
		.iter()
		.map(|header| ColumnDef {
			name: Ident {
				value: header.to_string(),
				quote_style: None,
			},
			data_type: DataType::Text,
			collation: None,
			options: vec![],
		})
		.collect();
	if headers.is_empty() {
		Ok(None)
	} else {
		Ok(Some(Schema {
			table_name: String::new(),
			column_defs,
		}))
	}
}
