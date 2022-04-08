use {
	crate::{glue::Context, parse_sql::Query, Glue, Result, Row, StorageInner, Value},
	serde::Serialize,
	sqlparser::ast::{SetVariableValue, Statement},
	std::convert::TryInto,
	thiserror::Error as ThisError,
};

#[derive(ThisError, Serialize, Debug, PartialEq)]
pub enum ExecuteError {
	#[error("query not supported")]
	QueryNotSupported,

	#[error("SET does not currently support columns, aggregates or subqueries")]
	MissingComponentsForSet,

	#[error("unsupported insert value type: {0}")]
	UnreachableUnsupportedInsertValueType(String),

	#[error("object not recognised")]
	ObjectNotRecognised,
	#[error("unimplemented")]
	Unimplemented,
	#[error("database already exists")]
	DatabaseExists(String),
	#[error("invalid file location")]
	InvalidFileLocation,
	#[error("invalid database location")]
	InvalidDatabaseLocation,

	#[error("table does not exist")]
	TableNotExists,

	#[error("column could not be found")]
	ColumnNotFound,
}

#[derive(Serialize, Debug, PartialEq)]
pub enum Payload {
	Success,
	Create,
	Insert(usize),
	Select {
		labels: Vec<String>,
		rows: Vec<Row>,
	},
	Delete(usize),
	Update(usize),
	DropTable,
	#[cfg(feature = "alter-table")]
	AlterTable,
	TruncateTable,
}

impl Glue {
	pub async fn execute_query(&mut self, statement: &Query) -> Result<Payload> {
		let Query(statement) = statement;

		match statement {
			//- Modification
			//-- Tables
			Statement::CreateTable {
				name,
				columns,
				if_not_exists,
				..
			} => self
				.create_table(name, columns, *if_not_exists)
				.await
				.map(|_| Payload::Create),
			Statement::Drop {
				object_type,
				names,
				if_exists,
				..
			} => self
				.drop(object_type, names, *if_exists)
				.await
				.map(|_| Payload::DropTable),
			#[cfg(feature = "alter-table")]
			Statement::AlterTable { name, operation } => self
				.alter_table(name, operation)
				.await
				.map(|_| Payload::AlterTable),
			Statement::Truncate { table_name, .. } => self
				.truncate(table_name)
				.await
				.map(|_| Payload::TruncateTable),
			Statement::CreateIndex {
				name,
				table_name,
				columns,
				unique,
				if_not_exists,
			} => self
				.create_index(table_name, name, columns, *unique, *if_not_exists)
				.await
				.map(|_| Payload::Create),

			//-- Rows
			Statement::Insert {
				table_name,
				columns,
				source,
				..
			} => self.insert(table_name, columns, source, false).await,
			Statement::Update {
				table,
				selection,
				assignments,
				// TODO
				from: _,
			} => self.update(table, selection, assignments).await,
			Statement::Delete {
				table_name,
				selection,
			} => self.delete(table_name, selection).await,

			//- Selection
			Statement::Query(query_value) => {
				let result = self.query(*query_value.clone()).await?;
				let (labels, rows) = result;
				let rows = rows.into_iter().map(Row).collect(); // I don't like this. TODO
				let payload = Payload::Select { labels, rows };
				Ok(payload)
			}

			//- Context
			Statement::SetVariable {
				variable, value, ..
			} => {
				let first_value = value.get(0).unwrap(); // Why might one want anything else?
				let value: Value = match first_value {
					SetVariableValue::Ident(..) => unimplemented!(),
					SetVariableValue::Literal(literal) => literal.try_into()?,
				};
				let name = variable.value.clone();
				self.get_mut_context()?.set_variable(name, value);
				Ok(Payload::Success)
			}

			Statement::ExplainTable { table_name, .. } => self.explain(table_name).await,

			Statement::CreateDatabase { .. } => unreachable!(), // Handled at Glue interface // TODO: Clean up somehow
			_ => Err(ExecuteError::QueryNotSupported.into()),
		}
	}
}
