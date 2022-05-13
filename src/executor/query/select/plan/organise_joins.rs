use {
	super::{Manual, Order, SelectItem},
	crate::{
		executor::{
			query::select::join::{JoinExecute, JoinPlan},
			types::ColumnInfo,
			PlannedRecipe,
		},
		Glue, Result,
	},
};

pub(crate) fn organise_joins(mut needed_joins: Vec<(usize, JoinPlan)>) -> Vec<(usize, JoinPlan)> {
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

	requested_joins
}
