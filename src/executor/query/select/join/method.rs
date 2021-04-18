use {
    super::{JoinError, JoinType},
    crate::{
        executor::{types::Row, PlannedRecipe},
        NullOrd, Result, Value,
    },
    std::{cmp::Ordering, fmt::Debug},
};

macro_rules! unwrap_or_break {
    ($unwrap: expr) => {
        match $unwrap {
            Some(value) => value,
            None => {
                break;
            }
        };
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
                let left_width = plane_rows.get(0).map(|row| row.len()).unwrap_or(0);
                let right_width = self_rows.get(0).map(|row| row.len()).unwrap_or(0);
                let mut used_right_indexes: Vec<usize> = vec![];
                let mut rows = plane_rows
                    .into_iter()
                    .map(|left_row| {
                        let mut inner_rows = self_rows
                            .iter()
                            .enumerate()
                            .map(|(index, right_row)| {
                                Ok(if recipe.confirm_join_constraint(&left_row, &right_row)? {
                                    if !used_right_indexes.iter().any(|used| used == &index) {
                                        used_right_indexes.push(index)
                                    };
                                    vec![join_parts(left_row.clone(), right_row.clone())]
                                } else {
                                    vec![]
                                })
                            })
                            .reduce(|all: Result<Vec<Row>>, set| {
                                let (mut all, set) = (all?, set?);
                                all.extend(set);
                                Ok(all)
                            }) // TODO: Improve
                            .unwrap_or(Ok(vec![]))?;
                        if inner_rows.is_empty() && join.includes_left() {
                            inner_rows
                                .push(join_parts(left_row.clone(), vec![Value::Null; right_width]))
                        }
                        Ok(inner_rows)
                    })
                    .try_fold(
                        vec![],
                        |mut result_rows: Vec<Row>, joined_rows_sets: Result<Vec<Row>>| {
                            joined_rows_sets.map(|joined_rows_sets| {
                                result_rows.extend(joined_rows_sets);
                                result_rows
                            })
                        },
                    )?; // TODO: Improve a lot!
                used_right_indexes.sort_unstable();
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
                    plane_rows.sort_unstable_by(|row_l, row_r| {
                        row_l
                            .get(plane_index)
                            .partial_cmp(&row_r.get(plane_index))
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
                    .into_iter();

                // TODO: These should be in seperate threads
                // TODO: !!! This is vulnerable to NULLs, it won't handle them properly
                if !self_trust_ordered {
                    self_rows.sort_unstable_by(|row_l, row_r| {
                        row_l
                            .get(plane_index)
                            .partial_cmp(&row_r.get(plane_index))
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
                    .into_iter();

                // Starting values
                let mut left_partition = left_partitions.next().unwrap(); // TODO: Handle
                let mut right_partition = right_partitions.next().unwrap(); // TODO: Handle

                // For later
                // Trust that rows have same len
                let left_len = left_partition.1.get(0).map(|row| row.len()).unwrap_or(0);
                let right_len = right_partition.1.get(0).map(|row| row.len()).unwrap_or(0);

                let mut left_results = vec![];
                let mut inner_results = vec![];
                let mut right_results = vec![];
                loop {
                    match left_partition.0.null_cmp(&right_partition.0) {
                        Some(Ordering::Less) => {
                            left_results.push(left_partition);
                            left_partition = unwrap_or_break!(left_partitions.next());
                        }
                        Some(Ordering::Equal) => {
                            inner_results.push((left_partition, right_partition));
                            left_partition = unwrap_or_break!(left_partitions.next());
                            right_partition = unwrap_or_break!(right_partitions.next());
                        }
                        None => {
                            left_results.push(left_partition);
                            left_partition = unwrap_or_break!(left_partitions.next());
                            right_results.push(right_partition);
                            right_partition = unwrap_or_break!(right_partitions.next());
                        }
                        Some(Ordering::Greater) => {
                            right_results.push(right_partition);
                            right_partition = unwrap_or_break!(right_partitions.next());
                        }
                    }
                }
                // In case any remain
                left_results.extend(left_partitions);
                right_results.extend(right_partitions);

                let left_rows = left_results
                    .into_iter()
                    .map(|(_, left_rows)| {
                        left_rows
                            .into_iter()
                            .map(|left| join_parts(left, vec![Value::Null; right_len]))
                            .collect::<Vec<Row>>()
                    })
                    .reduce(|mut all, set| {
                        all.extend(set);
                        all
                    })
                    .unwrap_or(vec![]);

                let mut inner_rows = inner_results
                    .into_iter()
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
                    .reduce(|mut all, set| {
                        all.extend(set);
                        all
                    })
                    .unwrap_or(vec![]);

                let right_rows = right_results
                    .into_iter()
                    .map(|(_, right_rows)| {
                        right_rows
                            .into_iter()
                            .map(|right| join_parts(vec![Value::Null; left_len], right))
                            .collect::<Vec<Row>>()
                    })
                    .reduce(|mut all, set| {
                        all.extend(set);
                        all
                    })
                    .unwrap_or(vec![]);

                println!("Left Rows:\n{:?}", left_rows);
                println!("Right Rows:\n{:?}", right_rows);

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
