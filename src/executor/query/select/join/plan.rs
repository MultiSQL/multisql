use {
    super::{JoinError, JoinExecute, JoinManual, JoinMethod, JoinType},
    crate::{
        executor::{
            fetch::fetch_columns,
            types::{ComplexColumnName, Table, TableWithAlias},
            Ingredient, MetaRecipe, Method, PlannedRecipe, Recipe,
        },
        store::Store,
        Result, Value,
    },
    std::{cmp::Ordering, fmt::Debug},
};

pub struct JoinPlan {
    pub table: Table,
    pub needed_tables: Vec<Table>,
    join_type: JoinType,
    columns: Vec<ComplexColumnName>,
    method: Option<JoinMethod>,
    unconverted_constraint: MetaRecipe,
}
impl PartialEq for JoinPlan {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}
impl Eq for JoinPlan {}
impl PartialOrd for JoinPlan {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.join_type.cmp(&other.join_type))
    }
}
impl Ord for JoinPlan {
    fn cmp(&self, other: &Self) -> Ordering {
        self.join_type.cmp(&other.join_type)
    }
}

impl JoinPlan {
    pub async fn new<'a, Key: 'static + Debug>(
        join_manual: JoinManual,
        storage: &'a dyn Store<Key>,
    ) -> Result<Self> {
        let JoinManual {
            table,
            constraint,
            join_type,
        } = join_manual;
        let columns = get_columns(storage, table.clone()).await?;
        Ok(Self {
            table: table.1,
            join_type,
            columns,
            method: None,
            unconverted_constraint: constraint,
            needed_tables: vec![],
        })
    }
    pub async fn new_and_columns<'a, Key: 'static + Debug>(
        join_manual: JoinManual,
        storage: &'a dyn Store<Key>,
    ) -> Result<(Self, Vec<ComplexColumnName>)> {
        let new = Self::new(join_manual, storage).await?;
        let columns = new.columns.clone();
        Ok((new, columns))
    }
    pub fn executor(self) -> JoinExecute {
        JoinExecute {
            table: self.table,
            method: self.method.unwrap(), // TODO: Handle. Not user based, API based. This should not be called until method is filled using decide_method.
            join_type: self.join_type,
        }
    }
    pub fn decide_method(&mut self, plane_columns: Vec<ComplexColumnName>) -> Result<()> {
        self.method = Some(match &self.unconverted_constraint.recipe {
            Recipe::Ingredient(Ingredient::Value(Value::Bool(true))) => JoinMethod::All,
            Recipe::Method(method) => match **method {
                Method::BinaryOperation(
                    operator,
                    Recipe::Ingredient(Ingredient::Column(index_l)),
                    Recipe::Ingredient(Ingredient::Column(index_r)),
                ) if operator == Value::eq => {
                    // TODO: Be more strict, ensure that one column is from self, and another from another.
                    let column_l = self
                        .unconverted_constraint
                        .meta
                        .columns
                        .get(index_l)
                        .ok_or(JoinError::Unreachable)?;
                    let column_r = self
                        .unconverted_constraint
                        .meta
                        .columns
                        .get(index_r)
                        .ok_or(JoinError::Unreachable)?;
                    let (self_index, plane_index) = if let Some(self_index) =
                        self.columns.iter().position(|column| column == column_l)
                    {
                        let plane_index = plane_columns
                            .iter()
                            .position(|column| column == column_r)
                            .ok_or(JoinError::Unreachable)?;
                        (self_index, plane_index)
                    } else {
                        let self_index = self
                            .columns
                            .iter()
                            .position(|column| column == column_r)
                            .ok_or(JoinError::Unreachable)?;
                        let plane_index = plane_columns
                            .iter()
                            .position(|column| column == column_l)
                            .ok_or(JoinError::Unreachable)?;
                        (self_index, plane_index)
                    };

                    let comparison_table = plane_columns
                        .get(plane_index)
                        .ok_or(JoinError::Unreachable)?
                        .table
                        .1
                        .clone();
                    self.needed_tables.push(comparison_table);

                    JoinMethod::ColumnEqColumn {
                        plane_trust_ordered: false,
                        plane_index,
                        self_trust_ordered: false,
                        self_index,
                    }
                }
                // TODO: Methods for:
                // (self)Column = (other)Column AND (self)Column = (other or otherother)Column
                // (self)Column = (other)Column OR (self)Column = (other or otherother)Column
                _ => {
                    let recipe =
                        PlannedRecipe::new(self.unconverted_constraint.clone(), &plane_columns)?;
                    let mut needed_tables = recipe.needed_column_indexes
                        .iter()
                        .filter_map(|index| {
                            plane_columns
                            .get(*index)/* if this doesn't find something, something has gone terribly wrong */
                            .map(|column| column.table.1.clone())
                            .filter(|table| table != &self.table)
                        }).collect::<Vec<String>>();
                    needed_tables.sort_unstable();
                    needed_tables.dedup();
                    self.needed_tables = needed_tables;
                    JoinMethod::General(recipe)
                }
            },
            _ => JoinMethod::Ignore,
        });
        Ok(())
    }
}

async fn get_columns<'a, Key: 'static + Debug>(
    storage: &'a dyn Store<Key>,
    table: TableWithAlias,
) -> Result<Vec<ComplexColumnName>> {
    Ok(fetch_columns(storage, table.1.as_str())
        .await?
        .into_iter()
        .map(|name| ComplexColumnName {
            table: table.clone(),
            name: name.value,
        })
        .collect::<Vec<ComplexColumnName>>())
}
