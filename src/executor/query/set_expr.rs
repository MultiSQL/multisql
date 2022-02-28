use {
	super::{select::select, QueryError},
	crate::{
		executor::{alter_row::insert, types::LabelsAndRows},
		macros::warning,
		result::Result,
		Context, MetaRecipe, Payload, RecipeUtilities, StorageInner, Value,
	},
	async_recursion::async_recursion,
	sqlparser::ast::{OrderByExpr, SetExpr, SetOperator, Statement},
};

#[async_recursion(?Send)]
pub async fn from_body(
	storages: &mut Vec<(String, &mut StorageInner)>,
	context: &mut Context,
	body: SetExpr,
	order_by: Vec<OrderByExpr>,
) -> Result<LabelsAndRows> {
	match body {
		SetExpr::Select(query) => {
			let (labels, rows) = select(&storages, &context, *query, order_by).await?;
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
		SetExpr::SetOperation {
			op,
			all,
			left,
			right,
		} => {
			use SetOperator::*;
			if !order_by.is_empty() {
				warning!(
					"set operations (UNION, EXCEPT & INTERSECT) do not currently support ordering"
				);
			}
			let (left_labels, left) = from_body(storages, context, *left, vec![]).await?;
			let (right_labels, right) = from_body(storages, context, *right, vec![]).await?;
			if left_labels.len() != right_labels.len() {
				return Err(QueryError::OperationColumnsMisaligned.into());
			}
			let mut rows = match op {
				Union => [left, right].concat(),
				Except => left
					.into_iter()
					.filter(|row| !right.contains(&row))
					.collect(),
				Intersect => left
					.into_iter()
					.filter(|row| right.contains(&row))
					.collect(),
			};
			if !all {
				rows.dedup();
			}
			Ok((left_labels, rows))
		}
		SetExpr::Insert(Statement::Insert {
			table_name,
			columns,
			source,
			..
		}) => {
			let inserted = insert(storages, context, &table_name, &columns, &source, true).await?;
			if let Payload::Select { labels, rows } = inserted {
				Ok((labels, rows.into_iter().map(|row| row.0).collect()))
			} else {
				unreachable!(); // TODO: Handle
			}
		}
		_ => Err(QueryError::QueryNotSupported.into()), // TODO: Other queries
	}
}
