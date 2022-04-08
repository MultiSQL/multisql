use std::sync::Mutex;

use crate::ExecuteError;
use {
	crate::{
		parse, parse_single, CSVSettings, Connection, Payload, Query, Result, Storage, Value,
		WIPError,
	},
	futures::executor::block_on,
	sqlparser::ast::{
		Expr, Ident, ObjectName, ObjectType, Query as AstQuery, SetExpr, Statement,
		Value as AstValue, Values,
	},
	std::{collections::HashMap, fmt::Debug},
};

mod database;
mod error;
mod payload;
mod select;

pub use error::InterfaceError;

pub(crate) type Variables = HashMap<String, Value>;

#[derive(Default, Debug, Clone)]
pub struct Context {
	pub variables: Variables,
	pub tables: HashMap<String, (Vec<String>, Vec<Vec<Value>>)>,
}
impl Context {
	pub fn set_variable(&mut self, name: String, value: Value) {
		self.variables.insert(name, value);
	}
	pub fn set_table(&mut self, name: String, data: (Vec<String>, Vec<Vec<Value>>)) {
		self.tables.insert(name, data);
	}
}

/// # Glue
/// Glue is *the* interface for interacting with MultiSQL; a Glue instance comprises any number of stores, each with their own identifier.
/// Once built, one will typically interact with Glue via queries.
///
/// There is a number of ways to deposit queries however, depending on the desired output:
/// - [`Glue::execute()`] -- Might be considered the most generic.
///     Replies with a [Result]<[Payload]>
///     (payload being the response from any type of query).
/// - [`Glue::execute_many()`] -- Same as `execute()` but will find any number of seperate queries in given text and provide a [Vec] in response.
/// - [`Glue::select_as_string()`] -- Provides data, only for `SELECT` queries, as [String]s (rather than [Value]s).
/// - [`Glue::select_as_json()`] -- Provides data, only for `SELECT` queries, as one big [String]; generally useful for webby interactions.
pub struct Glue {
	pub primary: String,
	databases: HashMap<String, Storage>,
	context: Mutex<Context>,
}

/// ## Creation of new interfaces
impl Glue {
	/// Creates a [Glue] instance with just one [Storage].
	pub fn new(name: String, database: Storage) -> Self {
		let mut databases = HashMap::new();
		databases.insert(name, database);
		Self::new_multi(databases)
	}
	/// Creates a [Glue] instance with access to all provided storages.
	/// Argument is: [Vec]<(Identifier, [Storage])>
	pub fn new_multi(databases: HashMap<String, Storage>) -> Self {
		let context = Mutex::new(Context::default());
		let primary = databases.keys().next().cloned().unwrap_or_default();
		Self {
			databases,
			context,
			primary,
		}
	}
	/// Merges existing [Glue] instances
	pub fn new_multi_glue(glues: Vec<Glue>) -> Self {
		glues
			.into_iter()
			.reduce(|mut main, other| {
				main.databases.extend(other.databases);
				main
			})
			.unwrap()
	}
	/// Merge existing [Glue] with [Vec] of other [Glue]s
	/// For example:
	/// ```
	/// use multisql::{SledStorage, Storage, Glue};
	/// let storage = SledStorage::new("data/example_location/example")
	///   .map(Storage::new_sled)
	///   .expect("Storage Creation Failed");
	/// let mut glue = Glue::new(String::from("main"), storage);
	///
	/// glue.execute_many("
	///   DROP TABLE IF EXISTS test;
	///   CREATE TABLE test (id INTEGER);
	///   INSERT INTO test VALUES (1),(2);
	///   SELECT * FROM test WHERE id > 1;
	/// ");
	///
	/// let other_storage = SledStorage::new("data/example_location/example_other")
	///   .map(Storage::new_sled)
	///   .expect("Storage Creation Failed");
	/// let mut other_glue = Glue::new(String::from("other"), other_storage);
	///
	/// glue.extend(vec![other_glue]);
	/// ```
	///
	pub fn extend(&mut self, glues: Vec<Glue>) {
		self.databases.extend(
			glues
				.into_iter()
				.reduce(|mut main, other| {
					main.databases.extend(other.databases);
					main
				})
				.unwrap()
				.databases,
		)
	}
}

/// Internal: Modify
impl Glue {
	/*pub(crate) fn take_context(&mut self) -> Result<Context> {
		self.context
			.take()
			.ok_or(InterfaceError::ContextUnavailable.into())
	}
	pub(crate) fn replace_context(&mut self, context: Context) {
		self.context.replace(context);
	}*/
	#[allow(dead_code)]
	fn set_context(&mut self, context: Context) {
		self.context = Mutex::new(context);
	}
}

impl Glue {
	pub fn into_connections(self) -> Vec<(String, Connection)> {
		self.databases
			.into_iter()
			.map(|(name, storage)| (name, storage.into_source()))
			.collect()
	}
}

/// ## Execute (Generic)
impl Glue {
	/// Will execute a single query.
	pub fn execute(&mut self, query: &str) -> Result<Payload> {
		let parsed_query =
			parse_single(query).map_err(|error| WIPError::Debug(format!("{:?}", error)))?;
		self.execute_parsed(parsed_query)
	}
	/// Will execute a set of queries.
	pub fn execute_many(&mut self, query: &str) -> Result<Vec<Payload>> {
		let parsed_queries =
			parse(query).map_err(|error| WIPError::Debug(format!("{:?}", error)))?;
		parsed_queries
			.into_iter()
			.map(|parsed_query| self.execute_parsed(parsed_query))
			.collect::<Result<Vec<Payload>>>()
	}
	/// Will execute a pre-parsed query (see [Glue::pre_parse()] for more).
	pub fn execute_parsed(&mut self, query: Query) -> Result<Payload> {
		if let Query(Statement::CreateDatabase {
			db_name,
			if_not_exists,
			location,
			..
		}) = query
		{
			let store_name = db_name.0[0].value.clone();
			return if self.databases.iter().any(|(store, _)| store == &store_name) {
				if if_not_exists {
					Ok(Payload::Success)
				} else {
					Err(ExecuteError::DatabaseExists(store_name).into())
				}
			} else {
				match location {
					None => Err(ExecuteError::InvalidDatabaseLocation.into()), // TODO: Memory
					Some(location) => {
						let store = if location.ends_with('/') {
							Connection::Sled(location).try_into()?
						} else if location.ends_with(".csv") {
							Connection::CSV(location, CSVSettings::default()).try_into()?
						} else if location.ends_with(".xlsx") {
							Connection::Sheet(location).try_into()?
						} else {
							return Err(ExecuteError::InvalidDatabaseLocation.into());
						};
						self.extend(vec![Glue::new(store_name, store)]);
						Ok(Payload::Success)
					}
				}
			};
		} else if let Query(Statement::Execute { name, parameters }) = query {
			return match name.value.as_str() {
				"FILE" => {
					if let Some(Ok(query)) = parameters.get(0).map(|path| {
						if let Expr::Value(AstValue::SingleQuotedString(path)) = path {
							std::fs::read_to_string(path).map_err(|_| ())
						} else {
							Err(())
						}
					}) {
						self.execute(&query)
					} else {
						Err(ExecuteError::InvalidFileLocation.into())
					}
				}
				_ => Err(ExecuteError::Unimplemented.into()),
			};
		} else if let Query(Statement::Drop {
			object_type: ObjectType::Schema, // FOR NOW! // TODO: sqlparser-rs#454
			if_exists,
			names,
			..
		}) = query
		{
			let database_name = names
				.get(0)
				.and_then(|name| name.0.get(0).map(|name| name.value.clone()))
				.ok_or(ExecuteError::ObjectNotRecognised)?;

			if self.databases.contains_key(&database_name) {
				self.databases.remove(&database_name);
			} else if !if_exists {
				return Err(ExecuteError::ObjectNotRecognised.into());
			}
			return Ok(Payload::Success);
		}

		block_on(self.execute_query(&query))
	}
	/// Provides a parsed query to execute later.
	/// Particularly useful if executing a small query many times as parsing is not (computationally) free.
	pub fn pre_parse(query: &str) -> Result<Vec<Query>> {
		parse(query).map_err(|error| WIPError::Debug(format!("{:?}", error)).into())
	}
}

/// ## Insert (`INSERT`)
impl Glue {
	pub fn insert_vec(
		&mut self,
		table_name: String,
		columns: Vec<String>,
		rows: Vec<Vec<Value>>,
	) -> Result<Payload> {
		// TODO: Make this more efficient and nicer by checking the way we execute
		let table_name = ObjectName(vec![Ident {
			value: table_name,
			quote_style: None,
		}]);
		let columns = columns
			.into_iter()
			.map(|name| Ident {
				value: name,
				quote_style: None,
			})
			.collect();
		let sqlparser_rows: Vec<Vec<Expr>> = rows
			.into_iter()
			.map(|row| {
				row.into_iter()
					.map(|cell| {
						Expr::Value(match cell {
							Value::Null => AstValue::Null,
							Value::Bool(value) => AstValue::Boolean(value),
							Value::I64(value) => AstValue::Number(value.to_string(), false),
							Value::F64(value) => AstValue::Number(value.to_string(), false),
							Value::Str(value) => AstValue::SingleQuotedString(value),
							_ => unimplemented!(),
						})
					})
					.collect()
			})
			.collect();
		let body = SetExpr::Values(Values(sqlparser_rows));
		let query = Query(Statement::Insert {
			table_name, // !
			columns,    // !
			source: Box::new(AstQuery {
				body, // !
				order_by: vec![],
				with: None,
				limit: None,
				offset: None,
				fetch: None,
				lock: None,
			}),
			after_columns: vec![],
			table: false,
			overwrite: false,
			or: None,
			partitioned: None,
			on: None,
		});
		self.execute_parsed(query)
	}
}
