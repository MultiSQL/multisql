use {serde::Serialize, std::fmt::Debug, thiserror::Error};

#[derive(Error, Serialize, Debug, PartialEq)]
pub enum AlterError {
	// CREATE TABLE
	#[error("table already exists: {0}")]
	TableAlreadyExists(String),

	#[error("already exists: {0}")]
	AlreadyExists(String),

	// ALTER TABLE
	#[cfg(feature = "alter-table")]
	#[error("unsupported alter table operation: {0}")]
	UnsupportedAlterTableOperation(String),

	// DROP
	#[error("drop type not supported: {0}")]
	DropTypeNotSupported(String),

	#[error("table does not exist: {0}")]
	TableNotFound(String),

	#[error("column {1} does not exist on table {0}")]
	ColumnNotFound(String, String),

	// validate column def
	#[error("unsupported data type: {0}")]
	UnsupportedDataType(String),

	#[error("unsupported column option: {0}")]
	UnsupportedColumnOption(String),

	#[error("unsupported number of index columns for new index '{0}'")]
	UnsupportedNumberOfIndexColumns(String),

	#[error("column '{0}' of data type '{1}' is unsupported for unique constraint")]
	UnsupportedDataTypeForUniqueColumn(String, String),

	#[error("column '{0}' of data type '{1}' is unsupported for auto increment constraint, only INTEGER is allowed")]
	UnsupportedDataTypeForAutoIncrementColumn(String, String),
}
