mod get_columns;
mod get_data;
mod get_rows;
mod select;
mod set_expr;

pub use select::{join::*, ManualError, PlanError, SelectError};
use {
	crate::{
		executor::types::LabelsAndRows, result::Result, Cast, Glue, MetaRecipe, RecipeUtilities,
		Value,
	},
	async_recursion::async_recursion,
	serde::Serialize,
	sqlparser::ast::{Cte, Query, SetExpr, TableAlias, With},
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
	#[error(
		"UNION/EXCEPT/INTERSECT columns misaligned, sides should have an equal number of columns"
	)]
	OperationColumnsMisaligned,
}

impl Glue {
	#[async_recursion(?Send)]
	pub async fn query(&mut self, query: Query) -> Result<LabelsAndRows> {
		let Query {
			body,
			order_by,
			limit,
			offset,
			with,
			// TODO (below)
			fetch: _,
			lock: _,
		} = query;

		let limit: Option<usize> = limit
			.map(|expression| {
				MetaRecipe::new(expression)?
					.simplify_by_context(&*self.get_context()?)?
					.confirm_or_err(QueryError::MissingComponentsForLimit.into())?
					.cast()
			})
			.transpose()?;
		let offset: Option<usize> = offset
			.map(|offset| {
				MetaRecipe::new(offset.value)?
					.simplify_by_context(&*self.get_context()?)?
					.confirm_or_err(QueryError::MissingComponentsForOffset.into())?
					.cast()
			})
			.transpose()?;

		if let Some(with) = with {
			let With {
				recursive: _, // Recursive not currently supported
				cte_tables,
			} = with;
			for cte in cte_tables.into_iter() {
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
				let data = self.query(query).await?;
				self.get_mut_context()?.set_table(name, data);
			}
		}

		let (mut labels, mut rows) = self.from_body(body, order_by).await?;

		if let Some(offset) = offset {
			rows.drain(0..offset);
		}
		if let Some(limit) = limit {
			rows.truncate(limit);
		}
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
	pub async fn no_cte_query(&self, query: Query) -> Result<LabelsAndRows> {
		let Query {
			body,
			order_by,
			limit,
			offset,
			with,
			// TODO (below)
			fetch: _,
			lock: _,
		} = query;

		let limit: Option<usize> = limit
			.map(|expression| {
				MetaRecipe::new(expression)?
					.simplify_by_context(&*self.get_context()?)?
					.confirm_or_err(QueryError::MissingComponentsForLimit.into())?
					.cast()
			})
			.transpose()?;
		let offset: Option<usize> = offset
			.map(|offset| {
				MetaRecipe::new(offset.value)?
					.simplify_by_context(&*self.get_context()?)?
					.confirm_or_err(QueryError::MissingComponentsForOffset.into())?
					.cast()
			})
			.transpose()?;

		let (mut labels, mut rows) = if let SetExpr::Select(query) = body {
			let (labels, rows) = self.select_query(*query, order_by).await?;
			(labels, rows)
		} else {
			unimplemented!()
		};

		if let Some(offset) = offset {
			rows.drain(0..offset);
		}
		if let Some(limit) = limit {
			rows.truncate(limit);
		}
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
}
