use {
	crate::{
		AlterError, CSVDatabaseError, DatabaseError, ExecuteError, FetchError, InterfaceError,
		JoinError, ManualError, MemoryDatabaseError, PlanError, QueryError, RecipeError, RowError,
		SelectError, SheetDatabaseError, TableError, ValidateError, ValueError, CreateError,
	},
	serde::Serialize,
	std::marker::{Send, Sync},
	thiserror::Error as ThisError,
};

#[derive(ThisError, Serialize, Debug, PartialEq)]
pub enum WIPError {
	#[error("TODO")]
	TODO,
	#[error("{0}")]
	Debug(String),
}

#[derive(ThisError, Serialize, Debug)]
pub enum Error {
	#[error(transparent)]
	#[serde(with = "stringify")]
	Database(#[from] Box<dyn std::error::Error>),

	#[cfg(feature = "odbc-database")]
	#[error(transparent)]
	#[serde(with = "stringify")]
	ODBC(#[from] odbc_api::Error),

	#[error(transparent)]
	Execute(#[from] ExecuteError),
	#[error(transparent)]
	Alter(#[from] AlterError),
	#[error(transparent)]
	Create(#[from] CreateError),
	#[error(transparent)]
	Fetch(#[from] FetchError),
	#[error(transparent)]
	Select(#[from] SelectError),
	#[error(transparent)]
	Row(#[from] RowError),
	#[error(transparent)]
	Table(#[from] TableError),
	#[error(transparent)]
	Value(#[from] ValueError),
	#[error(transparent)]
	Recipe(#[from] RecipeError),
	#[error(transparent)]
	Join(#[from] JoinError),
	#[error(transparent)]
	Plan(#[from] PlanError),
	#[error(transparent)]
	Manual(#[from] ManualError),
	#[error(transparent)]
	Query(#[from] QueryError),
	#[error(transparent)]
	Validate(#[from] ValidateError),
	#[error(transparent)]
	WIP(#[from] WIPError),
	#[error(transparent)]
	DatabaseImplementation(#[from] DatabaseError),
	#[error(transparent)]
	CSVDatabase(#[from] CSVDatabaseError),
	#[error(transparent)]
	SheetDatabase(#[from] SheetDatabaseError),
	#[error(transparent)]
	MemoryDatabase(#[from] MemoryDatabaseError),
	#[error(transparent)]
	Interface(#[from] InterfaceError),
}

unsafe impl Send for Error {}
unsafe impl Sync for Error {}

pub type Result<T> = std::result::Result<T, Error>;
pub type MutResult<T, U> = std::result::Result<(T, U), (T, Error)>;

impl PartialEq for Error {
	fn eq(&self, other: &Error) -> bool {
		use Error::*;

		match (self, other) {
			(Execute(l), Execute(r)) => l == r,
			(Alter(l), Alter(r)) => l == r,
			(Create(l), Create(r)) => l == r,
			(Fetch(l), Fetch(r)) => l == r,
			(Select(l), Select(r)) => l == r,
			(Row(l), Row(r)) => l == r,
			(Table(l), Table(r)) => l == r,
			(Value(l), Value(r)) => l == r,
			(Recipe(l), Recipe(r)) => l == r,
			(Join(l), Join(r)) => l == r,
			(Plan(l), Plan(r)) => l == r,
			(Manual(l), Manual(r)) => l == r,
			(Query(l), Query(r)) => l == r,
			(Validate(l), Validate(r)) => l == r,
			(WIP(l), WIP(r)) => l == r,
			(DatabaseImplementation(l), DatabaseImplementation(r)) => l == r,
			(CSVDatabase(l), CSVDatabase(r)) => l == r,
			(SheetDatabase(l), SheetDatabase(r)) => l == r,
			(Interface(l), Interface(r)) => l == r,
			(MemoryDatabase(l), MemoryDatabase(r)) => l == r,
			_ => false,
		}
	}
}

mod stringify {
	use {serde::Serializer, std::fmt::Display};

	pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
	where
		T: Display,
		S: Serializer,
	{
		serializer.collect_str(value)
	}
}
