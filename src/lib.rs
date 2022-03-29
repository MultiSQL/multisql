//! # MultiSQL
//!
//! `multisql` is a highly modular SQL database engine library written in Rust.
//! It enables flexible querying via Rust interfaces.
//!
//! ## Examples
//!
//! ```
//! use gluesql::{SledStorage, Storage, Glue};
//! fn main() {
//! 	let storage = SledStorage::new(&path)
//! 		.map(Storage::new_sled)
//! 		.expect("Create Storage");
//! 	let mut glue = Glue::new(String::from("main"), storage)
//!     
//! 	glue.execute_many("
//! 		DROP TABLE IF EXISTS test;
//! 		CREATE TABLE test (id INTEGER);
//! 		INSERT INTO test VALUES (1),(2);
//! 		SELECT * FROM test WHERE id > 1;
//! 	");
//! }
//! ```

pub use sqlparser as parser;

mod executor;
mod glue;
mod parse_sql;
mod storages;
mod utils;
mod data;
mod result;
mod store;

pub use {data::*, executor::*, glue::*, parse_sql::*, result::*, storages::*, store::*};

pub(crate) use utils::macros;
