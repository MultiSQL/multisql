use {crate::Error, sled::transaction::TransactionError, std::str, thiserror::Error as ThisError};

#[derive(ThisError, Debug)]
pub enum DatabaseError {
	#[error(transparent)]
	Sled(#[from] sled::Error),
	#[error(transparent)]
	Bincode(#[from] bincode::Error),
	#[error(transparent)]
	Str(#[from] str::Utf8Error),
}

impl From<DatabaseError> for Error {
	fn from(e: DatabaseError) -> Error {
		use DatabaseError::*;

		match e {
			Sled(e) => Error::Database(Box::new(e)),
			Bincode(e) => Error::Database(e),
			Str(e) => Error::Database(Box::new(e)),
		}
	}
}
impl From<sled::Error> for Error {
	fn from(e: sled::Error) -> Error {
		Error::Database(Box::new(e))
	}
}
impl From<bincode::Error> for Error {
	fn from(e: bincode::Error) -> Error {
		Error::Database(Box::new(e))
	}
}

impl From<str::Utf8Error> for Error {
	fn from(e: str::Utf8Error) -> Error {
		Error::Database(Box::new(e))
	}
}

impl From<TransactionError<Error>> for Error {
	fn from(error: TransactionError<Error>) -> Error {
		match error {
			TransactionError::Abort(error) => error,
			TransactionError::Storage(error) => DatabaseError::Sled(error).into(),
		}
	}
}

pub fn err_into<E>(error: E) -> Error
where
	E: Into<DatabaseError>,
{
	let error: DatabaseError = error.into();
	let error: Error = error.into();

	error
}
