use {
	super::{
		join::{JoinExecute, JoinPlan},
		Manual, Order, SelectItem,
	},
	crate::{
		executor::{types::ComplexColumnName, PlannedRecipe},
		Result, StorageInner,
	},
	futures::future::join_all,
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
	pub async fn new(
		storages: &Vec<(String, &mut StorageInner)>,
		select: Select,
		order_by: Vec<OrderByExpr>,
	) -> Result<Plan> {
		let Manual {
			joins,
			select_items,
			constraint,
			group_constraint,
			groups,
		} = Manual::new(select)?;

		let mut joins: Vec<JoinPlan> = join_all(
			joins
				.into_iter()
				.map(|join| JoinPlan::new(join, storages))
				.collect::<Vec<_>>(),
		)
		.await
		.into_iter()
		.collect::<Result<Vec<JoinPlan>>>()?;

		joins.sort_unstable();
		let table_columns = joins.iter().map(|join| join.columns.clone()).collect();
		let joins = joins
			.into_iter()
			.map(|mut join| {
				join.calculate_needed_tables(&table_columns);
				join
			})
			.enumerate()
			.collect();

		let mut needed_joins: Vec<(usize, JoinPlan)> = joins;
		let mut requested_joins: Vec<(usize, JoinPlan)> = vec![];
		let mut len_last: usize;
		let mut len = 0;
		loop {
			len_last = len;
			len = needed_joins.len();
			if needed_joins.is_empty() {
				break;
			}
			let needed_joins_iter = needed_joins.into_iter();
			needed_joins = vec![];
			needed_joins_iter.for_each(|(needed_index, join)| {
				if !join.needed_tables.iter().any(|needed_table_index| {
					!(&needed_index == needed_table_index
						|| requested_joins
							.iter()
							.any(|(requested_index, _)| needed_table_index == requested_index))
				}) {
					requested_joins.push((needed_index, join))
				} else {
					if len == len_last {
						// TODO
						panic!(
							"Impossible Join, table not present or tables require eachother: {:?}",
							join
						)
						// TODO: Handle
					}
					needed_joins.push((needed_index, join))
				}
			});
		}
		let columns = requested_joins
			.iter()
			.fold(vec![], |mut columns, (index, _)| {
				columns.extend(
					table_columns
						.get(*index)
						.expect("Something went very wrong")
						.clone(),
				);
				columns
			});

		let mut joins = requested_joins
			.into_iter()
			.map(|(_, join)| JoinExecute::new(join, &columns))
			.collect::<Result<Vec<JoinExecute>>>()?;

		if let Some(first) = joins.first_mut() {
			first.set_first_table()
		}

		let include_table = joins.len() != 1;
		let select_items = select_items
			.into_iter()
			.enumerate()
			.map(|(index, select_item)| {
				Ok(match select_item {
					SelectItem::Recipe(meta_recipe, alias) => {
						let recipe = PlannedRecipe::new(meta_recipe, &columns)?;
						let label =
							alias.unwrap_or(recipe.get_label(index, include_table, &columns));
						vec![(recipe, label)]
					}
					SelectItem::Wildcard(specifier) => {
						let specified_table = specifier
							.map(|specifier| specifier.get(0).map(|result| result.clone()))
							.flatten();
						let matches_table = |column: &ComplexColumnName| {
							specified_table
								.clone()
								.map(|specified_table| {
									column.table.name == specified_table
										|| column
											.table
											.alias
											.clone()
											.map(|alias| alias == specified_table)
											.unwrap_or(false)
								})
								.unwrap_or(true)
						};
						columns
							.iter()
							.enumerate()
							.filter_map(|(index, column)| {
								if matches_table(column) {
									Some((
										PlannedRecipe::of_index(index),
										if include_table {
											format!("{}.{}", column.table.name, column.name)
										} else {
											column.name.clone()
										},
									))
								} else {
									None
								}
							})
							.collect()
					}
				})
			})
			.collect::<Result<Vec<Vec<(PlannedRecipe, String)>>>>()? // TODO: Don't do this
			.into_iter()
			.reduce(|mut select_items, select_item_set| {
				select_items.extend(select_item_set);
				select_items
			})
			.ok_or(PlanError::UnreachableNoSelectItems)?;

		let (select_items, labels) = select_items.into_iter().unzip();

		let constraint = PlannedRecipe::new(constraint, &columns)?;
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
			group_constraint,
			groups,
			order_by,
			labels,
		})
	}
}
