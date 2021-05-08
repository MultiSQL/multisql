#![cfg(feature = "sled-storage")]
use {
	crate::{
		execute, parse, parse_single, Cast, ExecuteError, Payload, Query, Result, Row, Storage,
		StorageInner, Value, WIPError,
	},
	futures::executor::block_on,
	serde_json::{json, value::Value as JSONValue},
	sqlparser::ast::{
		Expr, Ident, ObjectName, Query as AstQuery, SetExpr, Statement, Value as AstValue, Values,
	},
	std::{collections::HashMap, fmt::Debug},
};

mod value;

pub(crate) type Variables = HashMap<String, Value>;

#[derive(Default, Debug)]
pub struct Context {
	pub variables: Variables,
}
impl Context {
	pub fn set_variable(&mut self, name: String, value: Value) {
		self.variables.insert(name, value);
	}
}

pub struct Glue {
	storages: Vec<(String, Storage)>,
	context: Option<Context>,
}

// New
impl Glue {
	pub fn new_multi(storages: Vec<(String, Storage)>) -> Self {
		let context = Some(Context::default());
		Self { storages, context }
	}
	pub fn new_multi_glue(glues: Vec<Glue>) -> Self {
		glues
			.into_iter()
			.reduce(|mut main, other| {
				main.storages.extend(other.storages);
				main
			})
			.unwrap()
	}
	pub fn new(name: String, storage: Storage) -> Self {
		Self::new_multi(vec![(name, storage)])
	}
}

// Modify
impl Glue {
	pub fn take_context(&mut self) -> Context {
		self.context
			.take()
			.expect("Unreachable: Context wasn't replaced!")
	}
	pub fn replace_context(&mut self, context: Context) {
		self.context.replace(context);
	}
	pub fn set_context(&mut self, context: Context) {
		self.context = Some(context);
	}
}

// Execute
impl Glue {
	pub fn execute_parsed(&mut self, query: Query) -> Result<Payload> {
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

	pub fn execute(&mut self, query: &str) -> Result<Payload> {
		let parsed_query =
			parse_single(query).map_err(|error| WIPError::Debug(format!("{:?}", error)))?;
		self.execute_parsed(parsed_query)
	}
	pub fn execute_many(&mut self, query: &str) -> Result<Vec<Payload>> {
		let parsed_queries =
			parse(query).map_err(|error| WIPError::Debug(format!("{:?}", error)))?;
		parsed_queries
			.into_iter()
			.map(|parsed_query| self.execute_parsed(parsed_query))
			.collect::<Result<Vec<Payload>>>()
	}
	pub fn pre_parse(query: &str) -> Result<Vec<Query>> {
		parse(query).map_err(|error| WIPError::Debug(format!("{:?}", error)).into())
	}

	#[cfg(feature = "expanded-api")]
	pub fn select_as_string(&mut self, query: &str) -> Result<Vec<Vec<String>>> {
		// TODO: Make this more efficient and not affect database
		if let Payload::Select { labels, rows } = self.execute(query)? {
			Ok(vec![labels]
				.into_iter()
				.chain(
					rows.into_iter()
						.map(|row| {
							row.0
								.into_iter()
								.map(|value| value.cast())
								.collect::<Result<Vec<String>>>()
						})
						.collect::<Result<Vec<Vec<String>>>>()?,
				)
				.collect())
		} else {
			Err(ExecuteError::QueryNotSupported.into())
		}
	}

	#[cfg(feature = "expanded-api")]
	pub fn select_as_json(&mut self, query: &str) -> Result<String> {
		// TODO: Make this more efficient and not affect database if not select by converting earlier
		if let Payload::Select { labels, rows } = self.execute(query)? {
			let array = JSONValue::Array(
				rows.into_iter()
					.map(|row| {
						JSONValue::Object(
							row.0
								.into_iter()
								.enumerate()
								.map(|(index, cell)| (labels[index].clone(), cell.into()))
								.collect::<serde_json::map::Map<String, JSONValue>>(),
						)
					})
					.collect(),
			);
			Ok(array.to_string())
		} else {
			Err(ExecuteError::QueryNotSupported.into())
		}
	}
	#[cfg(feature = "expanded-api")]
	pub fn select_as_json_with_headers(&mut self, query: &str) -> String {
		// TODO: Make this more efficient and not affect database if not select by converting earlier
		let mut result = || -> Result<_> {
			if let Payload::Select { labels, rows } = self.execute(query)? {
				let array = JSONValue::Array(
					rows.into_iter()
						.map(|row| {
							JSONValue::Object(
								row.0
									.into_iter()
									.enumerate()
									.map(|(index, cell)| (labels[index].clone(), cell.into()))
									.collect::<serde_json::map::Map<String, JSONValue>>(),
							)
						})
						.collect(),
				);
				Ok(json!({
					"labels": JSONValue::from(labels),
					"data": array
				}))
			} else {
				Err(ExecuteError::QueryNotSupported.into())
			}
		};
		match result() {
			Ok(result) => result,
			Err(error) => {
				println!("{:?}", error);
				json!({"error": error.to_string()})
			}
		}
		.to_string()
	}

	#[cfg(feature = "expanded-api")]
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
			}),
			after_columns: vec![],
			table: false,
			overwrite: false,
			or: None,
			partitioned: None,
		});
		self.execute_parsed(query)
	}
}

impl Payload {
	pub fn unwrap_rows(self) -> Vec<Row> {
		if let Payload::Select { rows, .. } = self {
			rows
		} else {
			panic!("Expected Select!")
		}
	}
}

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
