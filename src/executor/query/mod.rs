mod select;

pub use select::{join::*, ManualError, PlanError, SelectError};

use {
	crate::{
		executor::types::LabelsAndRows, macros::warning, result::Result, Cast, Context, MetaRecipe,
		RecipeUtilities, StorageInner, Value,
	},
	async_recursion::async_recursion,
	select::select,
	serde::Serialize,
	sqlparser::ast::{Cte, Query, SetExpr, TableAlias},
	thiserror::Error as ThisError,
};

const ENSURE_SIZE: bool = true;

#[derive(ThisError, Serialize, Debug, PartialEq)]
pub enum QueryError {
	#[error("query not supported")]
	QueryNotSupported,
	#[error("values does not support columns, aggregates or subqueries")]
	MissingComponentsForValues,
	#[error("limit does not support columns, aggregates or subqueries")]
	MissingComponentsForLimit,
	#[error("offset does not support columns, aggregates or subqueries")]
	MissingComponentsForOffset,
	#[error("expected values but found none")]
	NoValues,
}

#[async_recursion(?Send)]
pub async fn query(
	storages: &Vec<(String, &mut StorageInner)>,
	context: &Context,
	query: Query,
) -> Result<LabelsAndRows> {
	let Query {
		body,
		order_by,
		limit,
		offset,
		with,
		// TODO (below)
		fetch: _,
	} = query;
	let limit: Option<usize> = limit
		.map(|expression| {
			MetaRecipe::new(expression)?
				.simplify_by_context(context)?
				.confirm_or_err(QueryError::MissingComponentsForLimit.into())?
				.cast()
		})
		.transpose()?;
	let offset: Option<usize> = offset
		.map(|offset| {
			MetaRecipe::new(offset.value)?
				.simplify_by_context(context)?
				.confirm_or_err(QueryError::MissingComponentsForOffset.into())?
				.cast()
		})
		.transpose()?;

	let mut context = context.clone();
	if let Some(with) = with {
		for cte in with.cte_tables.into_iter() {
			let Cte {
				alias,
				query,
				from: _, // What is `from` for?
			} = cte;
			let TableAlias {
				name,
				columns: _, // TODO: Columns - Check that number is same and then rename labels
			} = alias;
			let name = name.value;
			let data = self::query(storages, &context, query).await?;
			context.set_table(name, data);
		}
	}
	let context = &context;

	let (mut labels, mut rows) = match body {
		SetExpr::Select(query) => {
			let (labels, rows) = select(storages, context, *query, order_by).await?;
			Ok((labels, rows))
		}
		SetExpr::Values(values) => {
			if !order_by.is_empty() {
				warning!("VALUES does not currently support ordering");
			}
			let values = values.0;
			values
				.into_iter()
				.map(|values_row| {
					values_row
						.into_iter()
						.map(|cell| {
							MetaRecipe::new(cell)?
								.simplify_by_context(context)?
								.confirm_or_err(QueryError::MissingComponentsForValues.into())
						})
						.collect::<Result<Vec<Value>>>()
				})
				.collect::<Result<Vec<Vec<Value>>>>()
				.map(|values| {
					(
						(0..values.get(0).map(|first_row| first_row.len()).unwrap_or(0))
							.map(|index| format!("unnamed_{}", index))
							.collect(),
						values,
					)
				})
		}
		_ => Err(QueryError::QueryNotSupported.into()), // TODO: Other queries
	}?;

	offset.map(|offset| rows.drain(0..offset));
	limit.map(|limit| rows.truncate(limit));
	if ENSURE_SIZE {
		let row_width = rows
			.iter()
			.map(|values_row| values_row.len())
			.max()
			.unwrap_or(0);
		if row_width > 0 {
			rows = rows
				.into_iter()
				.map(|mut row| {
					row.resize(row_width, Value::Null);
					row
				})
				.collect();
			labels.resize(row_width, String::new())
		};
	}
	Ok((labels, rows))
}
