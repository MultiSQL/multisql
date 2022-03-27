#[cfg(feature = "alter-table")]
mod alter_table;
mod create_table;
mod create_index;
mod drop;
mod error;
mod truncate;
mod validate;

use validate::validate;

#[cfg(feature = "alter-table")]
pub use alter_table::alter_table;
pub use create_table::create_table;
pub use create_index::create_index;
pub use drop::drop;
pub use error::AlterError;
pub use truncate::truncate;
