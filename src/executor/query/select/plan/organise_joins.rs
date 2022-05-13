use {
	crate::{
		executor::{
			query::select::join::{JoinManual, JoinPlan},
			types::ColumnInfo,
		},
		Glue, Result,
	},
	futures::future::join_all,
};

impl Glue {
	pub(crate) async fn arrange_joins(
		&self,
		joins: Vec<JoinManual>,
	) -> Result<(Vec<(usize, JoinPlan)>, Vec<ColumnInfo>)> {
		let mut joins: Vec<JoinPlan> = join_all(
			joins
				.into_iter()
				.map(|join| JoinPlan::new(join, self))
				.collect::<Vec<_>>(),
		)
		.await
		.into_iter()
		.collect::<Result<Vec<JoinPlan>>>()?;

		joins.sort_unstable();
		let table_columns = joins
			.iter()
			.map(|join| join.columns.clone())
			.collect::<Vec<Vec<ColumnInfo>>>();
		let joins = joins
			.into_iter()
			.map(|mut join| {
				join.calculate_needed_tables(&table_columns);
				join
			})
			.enumerate()
			.collect();

		let requested_joins = organise_joins(joins);

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

		Ok((requested_joins, columns))
	}
}

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
