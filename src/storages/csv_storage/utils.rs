use {
	super::CSVStorage,
	crate::{Result, WIPError},
	csv::{Reader, ReaderBuilder},
	std::fs::File,
};

pub(crate) fn csv_reader(store: &CSVStorage) -> Result<Reader<File>> {
	let reader = ReaderBuilder::new()
		.delimiter(store.csv_settings.delimiter)
		.from_path(store.path.as_str())
		.map_err(|error| WIPError::Debug(format!("{:?}", error)))?;
	Ok(reader)
}

/*pub(crate) fn csv_writer<T: Write>(store: &CSVStorage, init: T) -> Result<Writer<T>> {
	let writer = WriterBuilder::new().delimiter(store.csv_settings.delimiter).from_writer(init);

	Ok(writer)
}*/
