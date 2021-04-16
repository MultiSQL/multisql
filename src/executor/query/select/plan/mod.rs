use {
    super::{
        join::{JoinManual, JoinPlan},
        Manual, ManualInput, ManualOutput, SelectItem,
    },
    crate::{
        executor::{
            types::{ComplexColumnName, Row},
            MetaRecipe, PlannedRecipe, Recipe, SimplifyBy,
        },
        Result, Store, Value,
    },
    futures::future::join_all,
    serde::Serialize,
    sqlparser::ast::Select,
    std::fmt::Debug,
    thiserror::Error as ThisError,
};

pub struct Plan {
    pub input: PlanInput,
    pub output: ManualOutput,
}
pub struct PlanInput {
    pub joins: Vec<JoinPlan>,
    pub select_items: Vec<PlannedRecipe>,
    pub constraint: PlannedRecipe,
    pub groups: Vec<PlannedRecipe>,
}
pub struct RecipePlan {
    recipe: Recipe,
    column_indexes: Vec<usize>,
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
    pub async fn new<'a, Key: 'static + Debug>(
        storage: &'a dyn Store<Key>,
        select: Select,
    ) -> Result<Plan> {
        let Manual {
            input:
                ManualInput {
                    joins,
                    select_items,
                    constraint,
                    groups,
                },
            output,
        } = Manual::new(select)?;

        let (joins, columns): (Vec<JoinPlan>, Vec<Vec<ComplexColumnName>>) = join_all(
            joins
                .into_iter()
                .map(|join| JoinPlan::new_and_columns(join, storage))
                .collect::<Vec<_>>(),
        )
        .await
        .into_iter()
        .collect::<Result<Vec<(JoinPlan, Vec<ComplexColumnName>)>>>()?
        .into_iter()
        .unzip();

        let columns = columns
            .into_iter()
            .reduce(|mut all_columns, columns| {
                all_columns.extend(columns);
                all_columns
            })
            .ok_or(PlanError::UnreachableNoColumns)?;

        let mut joins = joins
            .into_iter()
            .map(|mut join| {
                join.decide_method(columns.clone())?;
                Ok(join)
            })
            .collect::<Result<Vec<JoinPlan>>>()?;

        joins.sort_unstable();
        let mut needed_joins = joins;
        let mut requested_joins: Vec<JoinPlan> = vec![];
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
            needed_joins_iter.for_each(|join| {
                if !join.needed_tables.iter().any(|needed_table| {
                    !requested_joins
                        .iter()
                        .any(|join| &join.table == needed_table)
                }) {
                    requested_joins.push(join)
                } else {
                    if len == len_last {
                        // TODO
                        panic!("Impossible Join, table not present or tables require eachother")
                        // TODO: Handle
                    }
                    needed_joins.push(join)
                }
            });
        }
        let joins = requested_joins;

        let select_items = select_items
            .into_iter()
            .map(|select_item| {
                Ok(match select_item {
                    SelectItem::Recipe(meta_recipe) => {
                        vec![PlannedRecipe::new(meta_recipe, &columns)?]
                    }
                    SelectItem::Wildcard(specifier) => {
                        let specified_table = specifier
                            .map(|specifier| specifier.get(0).map(|result| result.clone()))
                            .flatten();
                        let matches_table = |column: &ComplexColumnName| {
                            specified_table
                                .clone()
                                .map(|specified_table| {
                                    column.table.1 == specified_table
                                        || column
                                            .table
                                            .0
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
                                    Some(PlannedRecipe::of_index(index))
                                } else {
                                    None
                                }
                            })
                            .collect()
                    }
                })
            })
            .collect::<Result<Vec<Vec<PlannedRecipe>>>>()? // TODO: Don't do this
            .into_iter()
            .reduce(|mut select_items, select_item_set| {
                select_items.extend(select_item_set);
                select_items
            })
            .ok_or(PlanError::UnreachableNoSelectItems)?;
        let groups = groups
            .into_iter()
            .map(|group| PlannedRecipe::new(group, &columns))
            .collect::<Result<Vec<PlannedRecipe>>>()?;
        let constraint = PlannedRecipe::new(constraint, &columns)?;
        Ok(Plan {
            input: PlanInput {
                joins,
                select_items,
                constraint,
                groups,
            },
            output,
        })
    }
}
