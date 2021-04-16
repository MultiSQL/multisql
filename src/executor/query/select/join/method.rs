use {
    super::{JoinError, JoinType},
    crate::{
        executor::{types::Row, PlannedRecipe},
        Result,
    },
    std::cmp::Ordering,
};

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
            JoinMethod::General(recipe) => self_rows
                .into_iter()
                .map(|self_row| {
                    plane_rows
                        .iter()
                        .map(|plane_row| {
                            Ok(if recipe.confirm_join_constraint(plane_row, &self_row)? {
                                join.complete_join(plane_row.clone(), self_row.clone())
                            } else {
                                join.incomplete_join(plane_row.clone(), self_row.clone())
                            })
                        })
                        .try_fold(
                            vec![],
                            |mut result_rows, joined_row_set: Result<Vec<Row>>| {
                                result_rows.extend(joined_row_set?);
                                Ok(result_rows)
                            },
                        )
                })
                .try_fold(
                    vec![],
                    |mut result_rows: Vec<Row>, joined_rows_sets: Result<Vec<Row>>| {
                        joined_rows_sets.map(|joined_rows_sets| {
                            result_rows.extend(joined_rows_sets);
                            result_rows
                        })
                    },
                )?,
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
                } // TODO: These should be in seperate threads
                if !self_trust_ordered {
                    self_rows.sort_unstable_by(|row_l, row_r| {
                        row_l
                            .get(plane_index)
                            .partial_cmp(&row_r.get(plane_index))
                            .unwrap_or(Ordering::Equal)
                    });
                }
                let mut start_pos = 0;
                let mut result_rows = vec![];
                for self_row_index in 0..self_rows.len() {
                    let self_row = self_rows
                        .get(self_row_index)
                        .ok_or(JoinError::UnreachableCellNotFound)?;
                    let mut previous_same = false;
                    for plane_row_index in start_pos..plane_rows.len() {
                        let plane_row = plane_rows
                            .get(plane_row_index)
                            .ok_or(JoinError::UnreachableCellNotFound)?;
                        let same = plane_row.get(plane_index) == self_row.get(self_index);

                        let result = if same {
                            join.complete_join(plane_row.clone(), self_row.clone())
                        } else {
                            join.incomplete_join(plane_row.clone(), self_row.clone())
                        };

                        result_rows.extend(result);

                        if !same {
                            if previous_same {
                                break;
                            } else {
                                start_pos = plane_row_index + 1;
                            }
                        }
                        previous_same = same;
                    }
                }

                result_rows
            }
        })
    }
}
