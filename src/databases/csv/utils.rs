use {
	super::CSVDatabase,
	crate::{Result, WIPError},
	csv::{Reader, ReaderBuilder},
	std::fs::File,
};

pub(crate) fn csv_reader(store: &CSVDatabase) -> Result<Reader<File>> {
	let reader = ReaderBuilder::new()
		.delimiter(store.csv_settings.delimiter)
		.quoting(store.csv_settings.quoting)
		.buffer_capacity(8 * 500 * 1_000_000) // 500MB
		.from_path(store.path.as_str())
		.map_err(|error| WIPError::Debug(format!("{:?}", error)))?;
	Ok(reader)
}

/*pub(crate) fn csv_writer<T: Write>(store: &CSVDatabase, init: T) -> Result<Writer<T>> {
	let writer = WriterBuilder::new().delimiter(store.csv_settings.delimiter).from_writer(init);

	Ok(writer)
}*/
