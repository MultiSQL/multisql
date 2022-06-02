use {
	super::types::get_first_name,
	crate::{parse_sql::Query, Glue, Result, Row},
	serde::Serialize,
	sqlparser::ast::{ObjectType, Statement},
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
			Statement::CreateDatabase {
				db_name,
				if_not_exists,
				location,
				..
			} => {
				if !self.try_extend_from_path(
					db_name.0[0].value.clone(),
					location
						.clone()
						.ok_or(ExecuteError::InvalidDatabaseLocation)?,
				)? && !if_not_exists
				{
					Err(ExecuteError::DatabaseExists(db_name.0[0].value.clone()).into())
				} else {
					Ok(Payload::Success)
				}
			}
			//- Modification
			//-- Tables
			Statement::CreateTable {
				name,
				columns,
				if_not_exists,
				..
			} => self
				.ast_create_table(name, columns, *if_not_exists)
				.await
				.map(|_| Payload::Create),
			Statement::CreateView {
				name,
				query,
				or_replace,
				..
			} => self
				.ast_create_view(name, query, *or_replace)
				.await
				.map(|_| Payload::Create),
			Statement::Drop {
				object_type,
				names,
				if_exists,
				..
			} => match object_type {
				ObjectType::Schema => {
					// Schema for now // TODO: sqlparser-rs#454
					if !self.reduce(&get_first_name(names)?) && !if_exists {
						Err(ExecuteError::ObjectNotRecognised.into())
					} else {
						Ok(Payload::Success)
					}
				}
				object_type => self
					.ast_drop(object_type, names, *if_exists)
					.await
					.map(|_| Payload::DropTable),
			},
			#[cfg(feature = "alter-table")]
			Statement::AlterTable { name, operation } => self
				.ast_alter_table(name, operation)
				.await
				.map(|_| Payload::AlterTable),
			Statement::Truncate { table_name, .. } => self
				.ast_truncate(table_name)
				.await
				.map(|_| Payload::TruncateTable),
			Statement::CreateIndex {
				name,
				table_name,
				columns,
				unique,
				if_not_exists,
			} => self
				.ast_create_index(table_name, name, columns, *unique, *if_not_exists)
				.await
				.map(|_| Payload::Create),

			//-- Rows
			Statement::Insert {
				table_name,
				columns,
				source,
				..
			} => self.ast_insert(table_name, columns, source, false).await,
			Statement::Update {
				table,
				selection,
				assignments,
				// TODO
				from: _,
			} => self.ast_update(table, selection, assignments).await,
			Statement::Delete {
				table_name,
				selection,
			} => self.ast_delete(table_name, selection).await,

			//- Selection
			Statement::Query(query_value) => {
				let result = self.ast_query(*query_value.clone()).await?;
				let (labels, rows) = result;
				let rows = rows.into_iter().map(Row).collect(); // I don't like this. TODO
				let payload = Payload::Select { labels, rows };
				Ok(payload)
			}

			//- Context
			Statement::SetVariable {
				variable, value, ..
			} => self
				.set_variable(variable.into(), value)
				.await
				.map(|_| Payload::Success),

			Statement::ExplainTable { table_name, .. } => self.explain(table_name).await,

			Statement::Execute { name, parameters } => self.ast_procedure(name, parameters).await,
			_ => Err(ExecuteError::QueryNotSupported.into()),
		}
	}
}
