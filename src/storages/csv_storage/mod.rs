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
	csv::Reader,
	serde::Serialize,
	sqlparser::ast::{ColumnDef, DataType, Ident},
	std::{
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
	//file: File,
	schema: Option<Schema>,
	path: String,
}

impl AlterTable for CSVStorage {}
impl FullStorage for CSVStorage {}

impl Storage {
	pub fn new_csv(storage: CSVStorage) -> Self {
		Self::new(Box::new(storage))
	}
}
impl CSVStorage {
	/*pub fn of_file(file: File) -> Result<Self> {
		/*let reader =
			Reader::from_path(filename).map_err(|error| WIPError::Debug(format!("{:?}", error)))?;
		let writer =
			Writer::from_path(filename).map_err(|error| WIPError::Debug(format!("{:?}", error)))?;*/
		let schema = None;
		Ok(Self { file, schema })
	}*/
	pub fn new(path: &str) -> Result<Self> {
		let file = OpenOptions::new()
			.read(true)
			.write(true)
			.create(true)
			.open(path)
			.map_err(|error| WIPError::Debug(format!("{:?}", error)))?;

		let schema = discern_schema(file)?;
		//Self::of_file(file)
		Ok(Self {
			//    file,
			schema,
			path: path.to_string(),
		})
	}
}

fn discern_schema(file: File) -> Result<Option<Schema>> {
	let mut reader = Reader::from_reader(file);
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
