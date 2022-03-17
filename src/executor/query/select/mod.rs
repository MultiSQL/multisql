pub mod join;
mod manual;
mod order;
mod plan;

pub use {
	manual::{Manual, ManualError, SelectItem},
	order::Order,
	plan::{Plan, PlanError},
};

use {
	crate::{
		executor::{
			types::{LabelsAndRows, Row},
			PlannedRecipe,
		},
		macros::try_option,
		Context, RecipeUtilities, Result, StorageInner, Value,
	},
	futures::stream::{self, StreamExt, TryStreamExt},
	rayon::prelude::*,
	serde::Serialize,
	sqlparser::ast::{OrderByExpr, Select},
	thiserror::Error as ThisError,
};

#[derive(ThisError, Serialize, Debug, PartialEq)]
pub enum SelectError {
	#[error("aggregate groups not supported")]
	GrouperMayNotContainAggregate,

	#[error("an aggregate was probably used where not allowed")]
	FinalSolveFailure,

	#[error("HAVING does not yet support aggregates")]
	UnimplementedAggregateHaving,

	#[error("this should be impossible, please report")]
	UnreachableFinalSolveFailure,
	#[error("this should be impossible, please report")]
	Unreachable,
}

pub async fn select(
	storages: &Vec<(String, &mut StorageInner)>,
	context: &Context,
	query: Select,
	order_by: Vec<OrderByExpr>,
) -> Result<LabelsAndRows> {
	let Plan {
		joins,
		select_items,
		constraint,
		group_constraint,
		groups,
		order_by,
		labels,
	} = Plan::new(storages, context, query, order_by).await?;

	let rows = stream::iter(joins)
		.map(Ok)
		.try_fold(vec![], |rows, join| async {
			join.execute(storages, context, rows).await
		})
		.await?;

	let rows = order_by.execute(rows)?; // TODO: This should be done after filtering

	let selected_rows = rows
		.iter()
		.filter_map(|row| match constraint.confirm_constraint(row) {
			Ok(true) => Some(
				select_items
					.iter()
					.map(|selection| selection.clone().simplify_by_row(row))
					.collect::<Result<Vec<PlannedRecipe>>>()
					.map(|selection| (selection, row.clone())),
			),
			Ok(false) => None,
			Err(error) => Some(Err(error)),
		})
		.collect::<Result<Vec<(Vec<PlannedRecipe>, Row)>>>()?;
	let do_group = !groups.is_empty()
		|| select_items
			.iter()
			.any(|select_item| !select_item.aggregates.is_empty());

	let final_rows = if do_group {
		let groups = if groups.is_empty() {
			vec![PlannedRecipe::TRUE]
		} else {
			groups
		};

		let identified: Vec<(Vec<Value>, Option<PlannedRecipe>, Vec<PlannedRecipe>)> =
			selected_rows
				.into_par_iter()
				.filter_map(|(selected_row, row)| {
					let group_constraint =
						try_option!(group_constraint.clone().simplify_by_row(&row));
					let group_constraint = match group_constraint.as_solution() {
						Some(Value::Bool(true)) => None,
						Some(Value::Bool(false)) => return None,
						Some(_) => unreachable!(), // TODO: Handle
						None => Some(group_constraint),
					};
					let groupers = try_option!(groups
						.iter()
						.map(|group| {
							group
								.clone()
								.simplify_by_row(&row)?
								.confirm_or_err(SelectError::GrouperMayNotContainAggregate.into())
						})
						.collect::<Result<Vec<Value>>>());
					Some(Ok((groupers, group_constraint, selected_row)))
				})
				.collect::<Result<Vec<(Vec<Value>, Option<PlannedRecipe>, Vec<PlannedRecipe>)>>>()?; // TODO: Handle, Don't collect

		let accumulations: Result<Vec<(Vec<Value>, Option<PlannedRecipe>, Vec<PlannedRecipe>)>> =
			identified
				.into_par_iter()
				.map::<_, Result<_>>(|acc| Ok(vec![acc]))
				.try_reduce_with(accumulate)
				.unwrap_or(Ok(vec![(vec![], None, vec![PlannedRecipe::default()])])); // TODO: Improve

		accumulations?
			.into_par_iter()
			.map(|(_grouper, _group_constraint, vals)| {
				vals.into_iter()
					.map(|val| val.finalise_accumulation())
					.collect::<Result<Vec<Value>>>()
			})
			.collect::<Result<Vec<Vec<Value>>>>()?
			// TODO: Manage grouper and constraint
	} else {
		selected_rows
			.into_iter()
			.map(|(selection, _)| {
				selection
					.into_iter()
					.map(|selected| selected.confirm())
					.collect::<Result<Row>>()
			})
			.collect::<Result<Vec<Row>>>()?
	};

	Ok((labels, final_rows))
}

fn accumulate(
	mut rows_l: Vec<(Vec<Value>, Option<PlannedRecipe>, Vec<PlannedRecipe>)>,
	rows_r: Vec<(Vec<Value>, Option<PlannedRecipe>, Vec<PlannedRecipe>)>,
) -> Result<Vec<(Vec<Value>, Option<PlannedRecipe>, Vec<PlannedRecipe>)>> {
	rows_r
		.into_iter()
		.try_for_each::<_, Result<_>>(|row_r| {
			let (grouper, group_constraint, vals) = row_r;
			let group_index = rows_l.iter().position(|(group, _, _)| group == &grouper);
			if let Some(group_index) = group_index {
				/*rows_l[group_index].1.map(|constraint| {
					if let Some(group_constraint) = group_constraint {
						constraint.accumulate(group_constraint).unwrap() // TODO: Handle
					};
				});*/
				rows_l[group_index].2 = rows_l[group_index]
					.2
					.clone() // TODO: Don't clone
					.into_iter()
					.zip(vals.into_iter())
					.map(|(mut col, val)| {
						col.accumulate(val)?;
						Ok(col)
					})
					.collect::<Result<_>>()?;
			} else {
				rows_l.push((grouper, group_constraint, vals));
			}
			Ok(())
		})?;

	Ok(rows_l)
}
