mod organise_joins;
mod refine_item;
pub(crate) use refine_item::*;
use {
	super::{
		join::{JoinExecute},
		Manual, Order, SelectItem,
	},
	crate::{
		executor::{PlannedRecipe},
		Glue, Result,
	},
	serde::Serialize,
	sqlparser::ast::{OrderByExpr, Select},
	thiserror::Error as ThisError,
};

pub struct Plan {
	pub joins: Vec<JoinExecute>,
	pub select_items: Vec<PlannedRecipe>,
	pub constraint: PlannedRecipe,
	pub groups: Vec<PlannedRecipe>,
	pub group_constraint: PlannedRecipe,
	pub order_by: Order,
	pub labels: Vec<String>,
}

#[derive(ThisError, Serialize, Debug, PartialEq)]
pub enum PlanError {
	#[error("this should be impossible, please report")]
	UnreachableNoColumns,
	#[error("this should be impossible, please report")]
	UnreachableNoSelectItems,
	#[error("this should be impossible, please report")]
	Unreachable,
}

impl Plan {
	pub async fn new(glue: &Glue, select: Select, order_by: Vec<OrderByExpr>) -> Result<Plan> {
		let Manual {
			joins,
			select_items,
			constraint,
			group_constraint,
			groups,
		} = Manual::new(glue, select)?;

		let (requested_joins, columns) = glue.arrange_joins(joins).await?;

		let (constraint, mut index_filters) = PlannedRecipe::new_constraint(constraint, &columns)?;

		let mut joins = requested_joins
			.into_iter()
			.map(|(_, join)| {
				let index_filter = index_filters.remove(&join.table);
				JoinExecute::new(join, &columns, index_filter)
			})
			.collect::<Result<Vec<JoinExecute>>>()?;

		if let Some(first) = joins.first_mut() {
			first.set_first_table()
		}

		let include_table = joins.len() != 1;
		let select_items = refine_items(select_items, &columns, include_table)?;
		let (select_items, labels) = select_items.into_iter().unzip();

		let group_constraint = PlannedRecipe::new(group_constraint, &columns)?;
		let groups = groups
			.into_iter()
			.map(|group| PlannedRecipe::new(group, &columns))
			.collect::<Result<Vec<PlannedRecipe>>>()?;
		let order_by = Order::new(order_by, &columns)?;

		Ok(Plan {
			joins,
			select_items,
			constraint,
			groups,
			group_constraint,
			order_by,
			labels,
		})
	}
}
