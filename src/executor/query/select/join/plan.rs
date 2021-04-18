use {
    super::{JoinError, JoinExecute, JoinManual, JoinMethod, JoinType},
    crate::{
        executor::{
            fetch::fetch_columns,
            types::{ComplexColumnName, Table, TableWithAlias},
            MetaRecipe,
        },
        store::Store,
        Result,
    },
    std::{cmp::Ordering, fmt::Debug},
};

#[derive(Debug)]
pub struct JoinPlan {
    pub table: Table,
    pub columns: Vec<ComplexColumnName>,
    pub join_type: JoinType,
    pub constraint: MetaRecipe,
    pub needed_tables: Vec<usize>,
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
        let table = table.1;
        Ok(Self {
            table,
            join_type,
            columns,
            constraint,
            needed_tables: vec![],
        })
    }
    pub fn calculate_needed_tables(&mut self, table_columns: &Vec<Vec<ComplexColumnName>>) {
        self.needed_tables = table_columns
            .iter()
            .enumerate()
            .filter_map(|(index, columns)| {
                if columns.iter().any(|table_column| {
                    self.constraint
                        .meta
                        .columns
                        .iter()
                        .any(|constraint_column| table_column == constraint_column)
                }) {
                    Some(index)
                } else {
                    None
                }
            })
            .collect()
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
