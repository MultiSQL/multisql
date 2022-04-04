#![allow(clippy::option_map_unit_fn, clippy::fn_address_comparisons)]

//! # MultiSQL
//!
//! `multisql` is a highly modular SQL database engine library written in Rust.
//! It enables flexible querying via Rust interfaces.
//!
//! ## Examples
//!
//! ```
//! use multisql::{SledStorage, Storage, Glue};
//! fn main() {
//!   let storage = SledStorage::new("data/example_location/example")
//!     .map(Storage::new_sled)
//!     .expect("Storage Creation Failed");
//!   let mut glue = Glue::new(String::from("main"), storage);
//!     
//!   glue.execute_many("
//!     DROP TABLE IF EXISTS test;
//!     CREATE TABLE test (id INTEGER);
//!     INSERT INTO test VALUES (1),(2);
//!     SELECT * FROM test WHERE id > 1;
//!   ");
//! }
//! ```
//!
//! ## See also
//! - [Glue] -- Primary interface
//! - [Storage] -- Needed to build an interface
//! - [SledStorage] -- Most common type of storage/backend
//! - [Value] -- Value wrapper

pub use sqlparser as parser;

mod data;
mod executor;
mod glue;
mod parse_sql;
mod result;
mod storages;
mod store;
mod utils;

pub use {data::*, executor::*, glue::*, parse_sql::*, result::*, storages::*, store::*};

pub(crate) use utils::macros;
