//! # MultiSQL
//!
//! `multisql` is a highly modular SQL database engine library written in Rust.
//! It enables flexible querying via Rust interfaces.
//!
//! ## SQL Documentation
//!
//! You can find our SQL documentation at [multisql.org](https://multisql.org).
//!
//! ## Examples
//!
//! ```
//! use multisql::{Connection, Glue};
//! let storage = Connection::Sled(String::from("data/example_location/lib_example"))
//!   .try_into()
//!   .expect("Storage Creation Failed");
//! let mut glue = Glue::new(String::from("main"), storage);
//!
//! glue.execute_many("
//!   DROP TABLE IF EXISTS test;
//!   CREATE TABLE test (id INTEGER);
//!   INSERT INTO test VALUES (1),(2);
//!   SELECT * FROM test WHERE id > 1;
//! ");
//! ```
//!
//! ## See also
//! - [Glue] -- Primary interface
//! - [Storage] -- Needed to build an interface
//! - [SledStorage] -- Most common type of storage/backend
//! - [Value] -- Value wrapper

mod data;
mod database;
mod databases;
mod executor;
mod glue;
mod parse_sql;
mod result;
mod utils;

pub use {data::*, database::*, databases::*, executor::*, glue::*, parse_sql::*, result::*};

pub(crate) use utils::*;
