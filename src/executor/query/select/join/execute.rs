use {
    super::{JoinError, JoinMethod, JoinPlan, JoinType},
    crate::{
        executor::types::{ComplexColumnName, Row},
        Ingredient, MetaRecipe, Method, PlannedRecipe, Recipe, Result, StorageInner, Value,
    },
};

#[derive(Debug)]
pub struct JoinExecute {
    pub database: String,
    pub table: String,
    pub method: JoinMethod,
    pub join_type: JoinType,
}

impl JoinExecute {
    pub fn new(plan: JoinPlan, plane_columns: &Vec<ComplexColumnName>) -> Result<Self> {
        let JoinPlan {
            database,
            table,
            join_type,
            constraint,
            columns,
            ..
        } = plan;
        let method = decide_method(constraint, columns, plane_columns)?;
        Ok(Self {
            database,
            table,
            method,
            join_type,
        })
    }
    pub fn set_first_table(&mut self) {
        self.method = JoinMethod::FirstTable;
    }
    pub async fn get_rows<'a>(&self, storage: &StorageInner) -> Result<Vec<Row>> {
        storage
            .scan_data(self.table.as_str())
            .await?
            .map(|result| result.map(|(_, row)| row.0))
            .collect::<Result<Vec<Row>>>()
    }
    pub async fn execute<'a>(
        self,
        storages: &Vec<(String, &mut StorageInner)>,
        plane_rows: Vec<Row>,
    ) -> Result<Vec<Row>> {
        let storage = storages
            .into_iter()
            .find_map(|(name, storage)| {
                if name == &self.database {
                    Some(&**storage)
                } else {
                    None
                }
            })
            .or(storages.get(0).map(|(_, storage)| &**storage))
            .ok_or(JoinError::Unreachable)?;

        let rows = self.get_rows(storage).await?;
        self.method.run(&self.join_type, plane_rows, rows)
    }
}

fn decide_method(
    constraint: MetaRecipe,
    self_columns: Vec<ComplexColumnName>,
    plane_columns: &Vec<ComplexColumnName>,
) -> Result<JoinMethod> {
    Ok(match &constraint.recipe {
        Recipe::Ingredient(Ingredient::Value(Value::Bool(true))) => JoinMethod::All,
        Recipe::Method(method) => match **method {
            Method::BinaryOperation(
                operator,
                Recipe::Ingredient(Ingredient::Column(index_l)),
                Recipe::Ingredient(Ingredient::Column(index_r)),
            ) if operator == Value::eq => {
                // TODO: Be more strict, ensure that one column is from plane, and another from self.
                let column_l = constraint
                    .meta
                    .columns
                    .get(index_l)
                    .ok_or(JoinError::Unreachable)?;
                let column_r = constraint
                    .meta
                    .columns
                    .get(index_r)
                    .ok_or(JoinError::Unreachable)?;

                let (self_index, plane_index) = if let Some(self_index) =
                    self_columns.iter().position(|column| column == column_l)
                {
                    let plane_index = plane_columns
                        .iter()
                        .position(|column| column == column_r)
                        .ok_or(JoinError::Unreachable)?;
                    (self_index, plane_index)
                } else {
                    let self_index = self_columns
                        .iter()
                        .position(|column| column == column_r)
                        .ok_or(JoinError::Unreachable)?;
                    let plane_index = plane_columns
                        .iter()
                        .position(|column| column == column_l)
                        .ok_or(JoinError::Unreachable)?;
                    (self_index, plane_index)
                };

                JoinMethod::ColumnEqColumn {
                    plane_trust_ordered: false,
                    plane_index,
                    self_trust_ordered: false,
                    self_index,
                }
            }
            // TODO: Methods for:
            // (plan)Column = (other)Column AND (plan)Column = (other or otherother)Column
            // (plan)Column = (other)Column OR (plan)Column = (other or otherother)Column
            _ => JoinMethod::General(PlannedRecipe::new(constraint.clone(), plane_columns)?),
        },
        _ => JoinMethod::Ignore,
    })
}
