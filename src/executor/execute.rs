use {
	super::{
		alter_row::{delete, insert, update},
		alter_table::{create_index, create_table, drop, truncate},
		other::explain,
		query::query,
	},
	crate::{glue::Context, parse_sql::Query, Result, Row, StorageInner, Value},
	serde::Serialize,
	sqlparser::ast::{SetVariableValue, Statement},
	std::convert::TryInto,
	thiserror::Error as ThisError,
};

#[cfg(feature = "alter-table")]
use super::alter_table::alter_table;

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

pub async fn execute(
	mut storages: Vec<(String, &mut StorageInner)>,
	context: &mut Context,
	statement: &Query,
) -> Result<Payload> {
	let Query(statement) = statement;

	match statement {
		//- Modification
		//-- Tables
		Statement::CreateTable {
			name,
			columns,
			if_not_exists,
			..
		} => create_table(storages[0].1, name, columns, *if_not_exists)
			.await
			.map(|_| Payload::Create),
		Statement::Drop {
			object_type,
			names,
			if_exists,
			..
		} => drop(storages[0].1, object_type, names, *if_exists)
			.await
			.map(|_| Payload::DropTable),
		#[cfg(feature = "alter-table")]
		Statement::AlterTable { name, operation } => alter_table(storages[0].1, name, operation)
			.await
			.map(|_| Payload::AlterTable),
		Statement::Truncate { table_name, .. } => truncate(storages[0].1, table_name)
			.await
			.map(|_| Payload::TruncateTable),
		Statement::CreateIndex {
			name,
			table_name,
			columns,
			unique,
			if_not_exists,
		} => create_index(
			storages[0].1,
			table_name,
			name,
			columns,
			*unique,
			*if_not_exists,
		)
		.await
		.map(|_| Payload::Create),

		//-- Rows
		Statement::Insert {
			table_name,
			columns,
			source,
			..
		} => insert(&mut storages, context, table_name, columns, source, false).await,
		Statement::Update {
			table,
			selection,
			assignments,
			// TODO
			from: _,
		} => update(storages[0].1, context, table, selection, assignments).await,
		Statement::Delete {
			table_name,
			selection,
		} => delete(&mut storages, context, table_name, selection).await,

		//- Selection
		Statement::Query(query_value) => {
			let result = query(&mut storages, context, *query_value.clone()).await?;
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
			context.set_variable(name, value);
			Ok(Payload::Success)
		}

		Statement::ExplainTable { table_name, .. } => explain(&storages, table_name).await,

		Statement::CreateDatabase { .. } => unreachable!(), // Handled at Glue interface // TODO: Clean up somehow
		_ => Err(ExecuteError::QueryNotSupported.into()),
	}
}
