use {
	super::JoinType,
	crate::{
		executor::{types::Row, PlannedRecipe},
		macros::try_option,
		JoinError, NullOrd, Result, Value,
	},
	rayon::prelude::*,
	std::{cmp::Ordering, fmt::Debug},
};

macro_rules! unwrap_or_break {
	($unwrap: expr) => {
		match $unwrap {
			Some(value) => value,
			None => {
				break;
			}
		}
	};
}

#[derive(Debug)]
pub enum JoinMethod {
	General(PlannedRecipe),
	ColumnEqColumn {
		plane_index: usize,
		plane_trust_ordered: bool,
		self_index: usize,
		self_trust_ordered: bool,
	},
	All,
	FirstTable,
	Ignore,
}

impl JoinMethod {
	pub fn run(
		self,
		join: &JoinType,
		left_width: usize,
		right_width: usize,
		mut plane_rows: Vec<Row>,
		mut self_rows: Vec<Row>,
	) -> Result<Vec<Row>> {
		// Very crucuial to have performant, needs *a lot* of optimisation.
		Ok(match self {
			JoinMethod::Ignore => plane_rows,
			JoinMethod::FirstTable => self_rows,
			JoinMethod::All => self_rows
				.into_iter()
				.fold(vec![], |mut result_rows, self_row| {
					let joined_rows = plane_rows.clone().into_iter().map(|mut plane_row| {
						plane_row.extend(self_row.clone());
						plane_row
					});
					result_rows.extend(joined_rows);
					result_rows
				}),
			JoinMethod::General(recipe) => {
				let unfolded_rows = plane_rows
					.into_par_iter()
					.map(|left_row| {
						let inner_rows = self_rows
							.iter()
							.enumerate()
							.filter_map(|(index, right_row)| {
								if try_option!(recipe.confirm_join_constraint(&left_row, &right_row))
								{
									Some(Ok((
										index,
										join_parts(left_row.clone(), right_row.clone()),
									)))
								} else {
									None
								}
							})
							.collect::<Result<Vec<(usize, Row)>>>()?;
						Ok(if inner_rows.is_empty() && join.includes_left() {
							(
								vec![],
								vec![join_parts(left_row.clone(), vec![Value::Null; right_width])],
							)
						} else {
							inner_rows.into_iter().unzip()
						})
					})
					.collect::<Result<Vec<(Vec<usize>, Vec<Row>)>>>()?;
				let (mut used_right_indexes, mut rows): (Vec<usize>, Vec<Row>) = unfolded_rows
					.into_iter()
					.reduce(
						|mut all: (Vec<usize>, Vec<Row>), set: (Vec<usize>, Vec<Row>)| {
							all.0.extend(set.0);
							all.1.extend(set.1);
							all
						},
					)
					.unwrap_or((vec![], vec![]));
				used_right_indexes.par_sort_unstable();
				used_right_indexes.dedup();
				self_rows.iter().enumerate().for_each(|(index, row)| {
					if !used_right_indexes.iter().any(|used| used == &index)
						&& join.includes_right()
					{
						rows.push(join_parts(vec![Value::Null; left_width], row.clone()))
					}
				});
				rows
			}
			JoinMethod::ColumnEqColumn {
				plane_index,
				plane_trust_ordered,
				self_index,
				self_trust_ordered,
			} => {
				if !plane_trust_ordered {
					plane_rows.par_sort_unstable_by(|row_l, row_r| {
						row_l
							.get(plane_index)
							.and_then(|row_l| row_r.get(plane_index).and_then(|row_r| row_l.null_cmp(&row_r)))
							.unwrap_or(Ordering::Equal)
					});
				}

				// partition
				let mut left_partitions = plane_rows
					.into_iter()
					.fold(
						vec![],
						|mut partitions: Vec<(Value, Vec<Row>)>, row: Row| {
							let value = row.get(plane_index).unwrap().clone(); // TODO: Handle
							if let Some(last) = partitions.last_mut() {
								if last.0 == value {
									last.1.push(row);
								} else {
									partitions.push((value, vec![row]));
								}
								partitions
							} else {
								vec![(value, vec![row])]
							}
						},
					)
					.into_iter()
					.peekable();

				if !self_trust_ordered {
					self_rows.par_sort_unstable_by(|row_l, row_r| {
						row_l
							.get(self_index)
							.and_then(|row_l| row_r.get(self_index).and_then(|row_r| row_l.null_cmp(&row_r)))
							.unwrap_or(Ordering::Equal)
					});
				}

				// partition
				let mut right_partitions = self_rows
					.into_iter()
					.fold(
						vec![],
						|mut partitions: Vec<(Value, Vec<Row>)>, row: Row| {
							let value = row.get(self_index).unwrap().clone(); // TODO: Handle
							if let Some(last) = partitions.last_mut() {
								if last.0 == value {
									last.1.push(row);
								} else {
									partitions.push((value, vec![row]));
								}
								partitions
							} else {
								vec![(value, vec![row])]
							}
						},
					)
					.into_iter()
					.peekable();

				let mut left_results = vec![];
				let mut inner_results = vec![];
				let mut right_results = vec![];

				loop { // TODO: There's probably a better way to do this
					match unwrap_or_break!(left_partitions.peek())
						.0
						.null_cmp(&unwrap_or_break!(right_partitions.peek()).0)
					{
						Some(Ordering::Less) => {
							left_results
								.push(left_partitions.next().ok_or(JoinError::Unreachable)?);
						}
						Some(Ordering::Equal) => {
							inner_results.push((
								left_partitions.next().ok_or(JoinError::Unreachable)?,
								right_partitions.next().ok_or(JoinError::Unreachable)?,
							));
						}
						None => {
							left_results
								.push(left_partitions.next().ok_or(JoinError::Unreachable)?);
							right_results
								.push(right_partitions.next().ok_or(JoinError::Unreachable)?);
						}
						Some(Ordering::Greater) => {
							right_results
								.push(right_partitions.next().ok_or(JoinError::Unreachable)?);
						}
					}
				}
				// In case any remain
				left_results.extend(left_partitions);
				right_results.extend(right_partitions);

				let left_rows = left_results
					.into_par_iter()
					.map(|(_, left_rows)| {
						left_rows
							.into_iter()
							.map(|left| join_parts(left, vec![Value::Null; right_width]))
							.collect::<Vec<Row>>()
					})
					.reduce(
						|| vec![],
						|mut all, set| {
							all.extend(set);
							all
						},
					);

				let mut inner_rows = inner_results
					.into_par_iter()
					.map(|((_, left_rows), (_, right_rows))| {
						left_rows
							.into_iter()
							.map(|left| {
								right_rows
									.clone()
									.into_iter()
									.map(|right| join_parts(left.clone(), right))
									.collect()
							})
							.reduce(|mut all: Vec<Row>, set| {
								all.extend(set);
								all
							})
							.unwrap_or(vec![])
					})
					.reduce(
						|| vec![],
						|mut all: Vec<Row>, set| {
							all.extend(set);
							all
						},
					);

				let right_rows = right_results
					.into_par_iter()
					.map(|(_, right_rows)| {
						right_rows
							.into_iter()
							.map(|right| join_parts(vec![Value::Null; left_width], right))
							.collect::<Vec<Row>>()
					})
					.reduce(
						|| vec![],
						|mut all, set| {
							all.extend(set);
							all
						},
					);

				if join.includes_left() {
					inner_rows.extend(left_rows)
				};
				if join.includes_right() {
					inner_rows.extend(right_rows)
				};
				inner_rows
			}
		})
	}
}

fn join_parts(mut left: Vec<Value>, right: Vec<Value>) -> Vec<Value> {
	left.extend(right);
	left
}
