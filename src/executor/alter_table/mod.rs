mod alter_table;
mod create_index;
mod create_table;
mod drop;
mod error;
mod truncate;
mod validate;
pub use error::AlterError;
use validate::validate;
