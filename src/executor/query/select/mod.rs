pub mod join;
mod manual;
mod order;
mod plan;

use {
	crate::{
		executor::{
			types::{LabelsAndRows, Row},
			PlannedRecipe,
		},
		macros::try_option,
		Glue, RecipeUtilities, Result, Value,
	},
	futures::stream::{self, StreamExt, TryStreamExt},
	rayon::prelude::*,
	serde::Serialize,
	sqlparser::ast::{OrderByExpr, Select},
	thiserror::Error as ThisError,
};
pub use {
	manual::{Manual, ManualError, SelectItem},
	order::Order,
	plan::{Plan, PlanError},
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

impl Glue {
	pub async fn select(&mut self, plan: Plan) -> Result<LabelsAndRows> {
		let Plan {
			joins,
			select_items,
			constraint,
			group_constraint,
			groups,
			order_by,
			labels,
		} = plan;
		let rows = stream::iter(joins)
			.map(Ok)
			.try_fold(vec![], |rows, join| async {
				join.execute(self, rows).await
			})
			.await?;

		let rows = order_by.execute(rows)?; // TODO: This should be done after filtering

		let selected_rows =
			rows.into_par_iter()
				.filter_map(|row| match constraint.confirm_constraint(&row) {
					Ok(true) => Some(
						select_items
							.clone()
							.into_iter()
							.map(|selection| selection.simplify_by_row(&row))
							.collect::<Result<Vec<PlannedRecipe>>>()
							.map(|selection| (selection, row)),
					),
					Ok(false) => None,
					Err(error) => Some(Err(error)),
				});
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

			let accumulations: Vec<(Vec<Value>, Option<PlannedRecipe>, Vec<PlannedRecipe>)> =
				selected_rows
					.filter_map(|selection| {
						let (selected_row, row) = try_option!(selection);
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
								group.clone().simplify_by_row(&row)?.confirm_or_err(
									SelectError::GrouperMayNotContainAggregate.into(),
								)
							})
							.collect::<Result<Vec<Value>>>());
						Some(Ok((groupers, group_constraint, selected_row)))
					})
					.map::<_, Result<_>>(|acc| acc.map(|acc| vec![acc]))
					.try_reduce_with(accumulate)
					.unwrap_or(Ok(vec![]))?; // TODO: Improve

			accumulations
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
				.map(|selection| {
					selection.and_then(|(selection, _)| {
						selection
							.into_iter()
							.map(|selected| selected.confirm())
							.collect::<Result<Row>>()
					})
				})
				.collect::<Result<Vec<Row>>>()?
		};

		Ok((labels, final_rows))
	}
	pub async fn select_query(
		&mut self,
		query: Select,
		order_by: Vec<OrderByExpr>,
	) -> Result<LabelsAndRows> {
		let plan = Plan::new(self, query, order_by).await?;
		self.select(plan).await
	}
}

#[allow(clippy::type_complexity)] // TODO
fn accumulate(
	mut rows_l: Vec<(Vec<Value>, Option<PlannedRecipe>, Vec<PlannedRecipe>)>,
	rows_r: Vec<(Vec<Value>, Option<PlannedRecipe>, Vec<PlannedRecipe>)>,
) -> Result<Vec<(Vec<Value>, Option<PlannedRecipe>, Vec<PlannedRecipe>)>> {
	rows_r.into_iter().try_for_each::<_, Result<_>>(|row_r| {
		let (grouper, group_constraint, vals) = row_r;
		let group_index = rows_l.iter().position(|(group, _, _)| group == &grouper);
		let new_group = if let Some(group_index) = group_index {
			let (group_grouper, group_group_constraint, group_vals) =
				rows_l.swap_remove(group_index);
			/*rows_l[group_index].1.map(|constraint| {
				if let Some(group_constraint) = group_constraint {
					constraint.accumulate(group_constraint).unwrap() // TODO: Handle
				};
			});*/
			// TODO

			let group_vals = group_vals
				.into_iter()
				.zip(vals.into_iter())
				.map(|(mut col, val)| {
					col.accumulate(val)?;
					Ok(col)
				})
				.collect::<Result<_>>()?;
			(group_grouper, group_group_constraint, group_vals)
		} else {
			(grouper, group_constraint, vals)
		};
		rows_l.push(new_group);
		Ok(())
	})?;

	Ok(rows_l)
}
