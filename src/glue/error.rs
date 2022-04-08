use {serde::Serialize, std::fmt::Debug, thiserror::Error};

#[derive(Error, Serialize, Debug, PartialEq)]
pub enum InterfaceError {
	#[error("database not found")]
	DatabaseNotFound,
	#[error("context currently unavailable")]
	ContextUnavailable,
}
