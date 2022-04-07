#![cfg(feature = "sled-storage")]

use crate::ExecuteError;
use {
	crate::{
		execute, parse, parse_single, CSVSettings, Connection, Payload, Query, Result, Row,
		Storage, StorageInner, Value, WIPError,
	},
	futures::executor::block_on,
	sqlparser::ast::{
		Expr, Ident, ObjectName, ObjectType, Query as AstQuery, SetExpr, Statement,
		Value as AstValue, Values,
	},
	std::{collections::HashMap, fmt::Debug},
};

mod select;

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
	storages: Vec<(String, Storage)>,
	context: Option<Context>,
}

/// ## Creation of new interfaces
impl Glue {
	/// Creates a [Glue] instance with just one [Storage].
	pub fn new(name: String, storage: Storage) -> Self {
		Self::new_multi(vec![(name, storage)])
	}
	/// Creates a [Glue] instance with access to all provided storages.
	/// Argument is: [Vec]<(Identifier, [Storage])>
	pub fn new_multi(storages: Vec<(String, Storage)>) -> Self {
		let context = Some(Context::default());
		Self { storages, context }
	}
	/// Merges existing [Glue] instances
	pub fn new_multi_glue(glues: Vec<Glue>) -> Self {
		glues
			.into_iter()
			.reduce(|mut main, other| {
				main.storages.extend(other.storages);
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
		self.storages.extend(
			glues
				.into_iter()
				.reduce(|mut main, other| {
					main.storages.extend(other.storages);
					main
				})
				.unwrap()
				.storages,
		)
	}
}

/// Internal: Modify
impl Glue {
	pub(crate) fn take_context(&mut self) -> Context {
		self.context
			.take()
			.expect("Unreachable: Context wasn't replaced!")
	}
	pub(crate) fn replace_context(&mut self, context: Context) {
		self.context.replace(context);
	}
	#[allow(dead_code)]
	fn set_context(&mut self, context: Context) {
		self.context = Some(context);
	}
}

impl Glue {
	pub fn into_connections(self) -> Vec<(String, Connection)> {
		self.storages
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
			return if self.storages.iter().any(|(store, _)| store == &store_name) {
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

			let index = self
				.storages
				.iter()
				.enumerate()
				.find_map(|(index, (name, _))| (name == &database_name).then(|| index));
			if let Some(index) = index {
				self.storages.remove(index);
			} else if !if_exists {
				return Err(ExecuteError::ObjectNotRecognised.into());
			}
			return Ok(Payload::Success);
		}

		let mut storages: Vec<(String, Box<StorageInner>)> = self
			.storages
			.iter_mut()
			.map(|(name, storage)| (name.clone(), storage.take()))
			.collect();
		let give_storages: Vec<(String, &mut StorageInner)> = storages
			.iter_mut()
			.map(|(name, storage)| (name.clone(), &mut **storage))
			.collect();

		let mut context = self.take_context();

		let result = block_on(execute(give_storages, &mut context, &query));

		self.storages
			.iter_mut()
			.zip(storages)
			.for_each(|((_name, storage), (_name_2, taken))| storage.replace(taken));

		self.replace_context(context);

		result
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

impl Payload {
	// TODO: Move
	pub fn unwrap_rows(self) -> Vec<Row> {
		if let Payload::Select { rows, .. } = self {
			rows
		} else {
			panic!("Expected Select!")
		}
	}
}

// TODO: Move
/*
#[cfg(test)]
mod tests {
	use {
		crate::{CSVStorage, Glue, Payload, Row, SledStorage, Storage, Value},
		std::convert::TryFrom,
	};
	#[test]
	fn eq() {
		std::fs::remove_dir_all("data").unwrap();
		std::fs::create_dir("data").unwrap();
		let config = sled::Config::default()
			.path("data/using_config")
			.temporary(true);

		let sled = SledStorage::try_from(config).unwrap();
		let mut glue = Glue::new(String::from("sled"), Storage::new(Box::new(sled.clone())));
		assert_eq!(
			glue.execute(
					"CREATE TABLE api_test (id INTEGER PRIMARY KEY, name TEXT, nullable TEXT NULL, is BOOLEAN)",
			),
			Ok(Payload::Create)
		);
		assert_eq!(
			glue.execute("INSERT INTO api_test (id, name, nullable, is) VALUES (1, 'test1', 'not null', TRUE), (2, 'test2', NULL, FALSE)"),
			Ok(Payload::Insert(2))
		);

		assert_eq!(
			glue.execute("SELECT id, name, is FROM api_test"), // Not selecting NULL because NULL != NULL. TODO: Expand this test so that NULL == NULL
			Ok(Payload::Select {
				labels: vec![String::from("id"), String::from("name"), String::from("is")],
				rows: vec![
					Row(vec![
						Value::I64(1),
						Value::Str(String::from("test1")),
						Value::Bool(true)
					]),
					Row(vec![
						Value::I64(2),
						Value::Str(String::from("test2")),
						Value::Bool(false)
					])
				]
			})
		);
		#[cfg(feature = "expanded-api")]
		assert_eq!(
			glue.select_as_string("SELECT * FROM api_test"),
			Ok(vec![
				vec![
					String::from("id"),
					String::from("name"),
					String::from("nullable"),
					String::from("is")
				],
				vec![
					String::from("1"),
					String::from("test1"),
					String::from("not null"),
					String::from("TRUE")
				],
				vec![
					String::from("2"),
					String::from("test2"),
					String::from("NULL"),
					String::from("FALSE")
				]
			])
		);

		#[cfg(feature = "expanded-api")]
		assert_eq!(
			glue.select_as_json("SELECT * FROM api_test"),
			Ok(String::from(
				r#"[{"id":1,"is":true,"name":"test1","nullable":"not null"},{"id":2,"is":false,"name":"test2","nullable":null}]"#
			))
		);

		use crate::Cast;

		let test_value: Result<String, _> = Value::Str(String::from("test")).cast();
		assert_eq!(test_value, Ok(String::from("test")));
		let test_value: Result<String, _> = (Value::Str(String::from("test")).clone()).cast();
		assert_eq!(test_value, Ok(String::from("test")));
		let test_value: Result<String, _> = Value::I64(1).cast();
		assert_eq!(test_value, Ok(String::from("1")));
		let test_value: Result<String, _> = (Value::I64(1).clone()).cast();
		assert_eq!(test_value, Ok(String::from("1")));

		assert_eq!(
			glue.execute("CREATE TABLE api_insert_vec (name TEXT, rating FLOAT)"),
			Ok(Payload::Create)
		);

		#[cfg(feature = "expanded-api")]
		assert_eq!(
			glue.insert_vec(
				String::from("api_insert_vec"),
				vec![String::from("name"), String::from("rating")],
				vec![vec![Value::Str(String::from("test")), Value::F64(1.2)]]
			),
			Ok(Payload::Insert(1))
		);

		assert_eq!(
			glue.execute("SELECT * FROM api_insert_vec"),
			Ok(Payload::Select {
				labels: vec![String::from("name"), String::from("rating")],
				rows: vec![Row(vec![Value::Str(String::from("test")), Value::F64(1.2)])]
			})
		);

		#[cfg(feature = "expanded-api")]
		assert_eq!(
			glue.insert_vec(
				String::from("api_insert_vec"),
				vec![String::from("name"), String::from("rating")],
				vec![
					vec![Value::Str(String::from("test2")), Value::F64(1.3)],
					vec![Value::Str(String::from("test3")), Value::F64(1.0)],
					vec![Value::Str(String::from("test4")), Value::F64(100000.94)]
				]
			),
			Ok(Payload::Insert(3))
		);

		assert_eq!(
			glue.execute("SELECT * FROM api_insert_vec"),
			Ok(Payload::Select {
				labels: vec![String::from("name"), String::from("rating")],
				rows: vec![
					Row(vec![Value::Str(String::from("test")), Value::F64(1.2)]),
					Row(vec![Value::Str(String::from("test2")), Value::F64(1.3)]),
					Row(vec![Value::Str(String::from("test3")), Value::F64(1.0)]),
					Row(vec![
						Value::Str(String::from("test4")),
						Value::F64(100000.94)
					])
				]
			})
		);

		// Multi Glue
		let csv_a = CSVStorage::new("data/using_config_a.csv").unwrap();
		let csv_b = CSVStorage::new("data/using_config_b.csv").unwrap();
		let _multi_glue_type_one = Glue::new_multi(vec![
			(String::from("sled"), Storage::new(Box::new(sled.clone()))),
			(String::from("csv"), Storage::new(Box::new(csv_a))),
		]);
		let mut csv_glue = Glue::new(String::from("csv"), Storage::new(Box::new(csv_b)));

		assert_eq!(
			csv_glue.execute("CREATE TABLE data (name TEXT, rating TEXT)"),
			Ok(Payload::Create)
		);

		assert_eq!(
			csv_glue.execute(
				r#"
				INSERT INTO
					data (
						name,
						rating
					)
				VALUES (
					'test2',
					'30.1'
				), (
					'test3',
					'0.1'
				)
			"#
			),
			Ok(Payload::Insert(2))
		);

		let mut multi_glue = Glue::new_multi_glue(vec![glue, csv_glue]);

		assert_eq!(
			multi_glue.execute(
				r#"
				SELECT
					*
				FROM
					sled.api_insert_vec
					INNER JOIN csv.data
						ON sled.api_insert_vec.name = csv.data.name
			"#
			),
			Ok(Payload::Select {
				labels: vec![
					String::from("api_insert_vec.name"),
					String::from("api_insert_vec.rating"),
					String::from("data.name"),
					String::from("data.rating")
				],
				rows: vec![
					Row(vec![
						Value::Str(String::from("test2")),
						Value::F64(1.3),
						Value::Str(String::from("test2")),
						Value::Str(String::from("30.1"))
					]),
					Row(vec![
						Value::Str(String::from("test3")),
						Value::F64(1.0),
						Value::Str(String::from("test3")),
						Value::Str(String::from("0.1"))
					])
				]
			})
		);
	}
}
*/
