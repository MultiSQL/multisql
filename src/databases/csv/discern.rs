use {
	super::string_record_to_row,
	crate::{
		data::Schema, CSVDatabaseError, CSVSettings, Cast, Column, Result, Value,
		ValueType,
	},
	csv::{Reader, ReaderBuilder, StringRecord},
	std::{
		default::Default,
		fs::{File},
	},
};

impl CSVSettings {
	pub(crate) fn new_reader(&self, file: File) -> Reader<File> {
		ReaderBuilder::new()
			.delimiter(self.delimiter)
			.has_headers(self.has_header.unwrap_or(true))
			.from_reader(file)
	}
	pub(crate) fn discern_header(&mut self, header: &StringRecord) -> Vec<String> {
		let header = string_record_to_row(header);

		let has_header = if let Some(has_header) = self.has_header {
			has_header
		} else {
			let has_header = !header
				.iter()
				.map(ValueType::from)
				.any(|vt| vt != ValueType::Str);
			self.has_header = Some(has_header);
			has_header
		};

		if has_header {
			header
				.into_iter()
				.map(Cast::cast)
				.collect::<Result<Vec<String>>>()
				.unwrap()
		} else {
			header
				.into_iter()
				.enumerate()
				.map(|(index, _)| format!("column_{}", index))
				.collect()
		}
	}
	pub(crate) fn discern_schema(&mut self, file: File) -> Result<Option<Schema>> {
		let mut reader = self.new_reader(file);
		let header = reader
			.headers()
			.map_err(|error| CSVDatabaseError::HeaderError(format!("{:?}", error)))?;
		let header = self.discern_header(header);
		if header.is_empty() {
			return Ok(None);
		}
		let value_types = self.discern_types(reader);

		let column_defs = header
			.into_iter()
			.zip(value_types)
			.map(|(header, value_type)| {
				let mut column = Column::default();
				column.name = header;
				column.data_type = value_type;
				column
			})
			.collect();
		Ok(Some(Schema {
			table_name: String::new(),
			column_defs,
			indexes: vec![],
		}))
	}
	pub(crate) fn discern_types(&self, reader: Reader<File>) -> Vec<ValueType> {
		let sample = reader
			.into_records()
			.take(self.sample_rows)
			.map(|record| string_record_to_row(&record.unwrap()))
			.collect();
		discern_types_from_sample(sample)
	}
}

pub(crate) fn discern_types_from_sample(sample: Vec<Vec<Value>>) -> Vec<ValueType> {
	let mut types = sample
		.into_iter()
		.map(|row| row.iter().map(ValueType::from).collect());
	let first_types = types.next().unwrap();
	types.fold(first_types, |mut out_types, row_types| {
		out_types
			.iter_mut()
			.zip(row_types)
			.for_each(|(out_type, row_type)| {
				if out_type != &row_type {
					*out_type = ValueType::Any
				}
			});
		out_types
	})
}
