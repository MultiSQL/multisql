use std::sync::Mutex;

use crate::ExecuteError;
use {
	crate::{
		parse, parse_single, CSVSettings, Connection, Payload, Query, Result, Storage, Value,
		WIPError,
	},
	futures::executor::block_on,
	sqlparser::ast::{
		Expr, Ident, ObjectName, Query as AstQuery, SetExpr, Statement,
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
	/// glue.extend_many_glues(vec![other_glue]);
	/// ```
	///
	pub fn extend_many_glues(&mut self, glues: Vec<Glue>) {
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
	pub fn extend_glue(&mut self, glue: Glue) {
		self.databases.extend(glue.databases)
	}

	/// Extend using a ~~[Path]~~ [String] which represents a path
	/// Guesses the type of database based on the extension
	pub fn try_extend_from_path(
		&mut self,
		database_name: String,
		database_path: String,
	) -> Result<bool> {
		let connection = if database_path.ends_with('/') {
			Connection::Sled(database_path)
		} else if database_path.ends_with(".csv") {
			Connection::CSV(database_path, CSVSettings::default())
		} else if database_path.ends_with(".xlsx") {
			Connection::Sheet(database_path)
		} else {
			return Err(ExecuteError::InvalidDatabaseLocation.into());
		};
		let database = connection.try_into()?;
		Ok(self.extend(database_name, database))
	}

	/// Extend [Glue] by single database
	pub fn extend(&mut self, database_name: String, database: Storage) -> bool {
		let database_present = self.databases.contains_key(&database_name);
		if !database_present {
			self.databases.insert(database_name, database);
		}
		database_present
	}

	/// Opposite of [Glue::extend], removes database
	/// Returns [bool] of whether the database was present
	pub fn reduce(&mut self, database_name: &String) -> bool {
		let database_present = self.databases.contains_key(database_name);
		if database_present {
			self.databases.remove(database_name);
		}
		database_present
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
