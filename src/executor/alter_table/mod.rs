#[cfg(feature = "alter-table")]
#[allow(clippy::module_inception)] // TODO
mod alter_table;
mod create_index;
mod create_table;
mod drop;
mod error;
mod truncate;
mod validate;
use validate::validate;

#[cfg(feature = "alter-table")]
pub use alter_table::alter_table;
pub use {
	create_index::create_index, create_table::create_table, drop::drop, error::AlterError,
	truncate::truncate,
};
