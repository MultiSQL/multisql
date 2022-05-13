use {serde::Serialize, std::fmt::Debug, thiserror::Error};

#[derive(Error, Serialize, Debug, PartialEq)]
pub enum CreateError {
	#[error("already exists: {0}")]
	AlreadyExists(String),
}
