use {
	super::{
		alter_row::{insert, update},
		alter_table::{create_table, drop, truncate},
		query::query,
		types::ComplexColumnName,
	},
	crate::{
		data::{get_name, Schema},
		glue::Context,
		parse_sql::Query,
		MetaRecipe, PlannedRecipe, Result, Row, StorageInner, Value,
	},
	serde::Serialize,
	sqlparser::ast::{ColumnDef, SetVariableValue, Statement},
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
		} => update(storages[0].1, &context, table, selection, assignments).await,
		Statement::Delete {
			table_name,
			selection,
		} => {
			let table_name = get_name(&table_name)?;
			let Schema { column_defs, .. } = storages[0]
				.1
				.fetch_schema(table_name)
				.await?
				.ok_or(ExecuteError::TableNotExists)?;

			let columns = column_defs
				.clone()
				.into_iter()
				.map(|column_def| {
					let ColumnDef { name, .. } = column_def;
					ComplexColumnName::of_name(name.value)
				})
				.collect();
			let filter = selection
				.clone()
				.map(|selection| {
					PlannedRecipe::new(
						MetaRecipe::new(selection)?.simplify_by_context(context)?,
						&columns,
					)
				})
				.unwrap_or(Ok(PlannedRecipe::TRUE))?;

			let keys = storages[0]
				.1
				.scan_data(table_name)
				.await?
				.filter_map(|row_result| {
					let (key, row) = match row_result {
						Ok(keyed_row) => keyed_row,
						Err(error) => return Some(Err(error)),
					};

					let row = row.0;

					let confirm_constraint = filter.confirm_constraint(&row.clone());
					match confirm_constraint {
						Ok(true) => Some(Ok(key)),
						Ok(false) => None,
						Err(error) => Some(Err(error)),
					}
				})
				.collect::<Result<Vec<Value>>>()?;

			let num_keys = keys.len();

			storages[0]
				.1
				.delete_data(keys)
				.await
				.map(|_| Payload::Delete(num_keys))
		}

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
		_ => Err(ExecuteError::QueryNotSupported.into()),
	}
}
