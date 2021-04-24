use {
    super::{JoinManual, JoinType},
    crate::{
        executor::{
            fetch::fetch_columns,
            types::{ComplexColumnName, ComplexTableName},
            MetaRecipe,
        },
        JoinError, Result, StorageInner,
    },
    std::cmp::Ordering,
};

#[derive(Debug)]
pub struct JoinPlan {
    pub database: String,
    pub table: String,
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
    pub async fn new<'a>(
        join_manual: JoinManual,
        storages: &Vec<(String, &mut StorageInner)>,
    ) -> Result<Self> {
        let JoinManual {
            table,
            constraint,
            join_type,
        } = join_manual;
        let columns = get_columns(storages, table.clone()).await?;
        let ComplexTableName {
            database,
            name: table,
            ..
        } = table;
        Ok(Self {
            database,
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

async fn get_columns(
    storages: &Vec<(String, &mut StorageInner)>,
    table: ComplexTableName,
) -> Result<Vec<ComplexColumnName>> {
    let storage = storages
        .into_iter()
        .find_map(|(name, storage)| {
            if name == &table.database {
                Some(&**storage)
            } else {
                None
            }
        })
        .or(storages.get(0).map(|(_, storage)| &**storage))
        .ok_or(JoinError::TableNotFound(table.clone()))?;

    Ok(fetch_columns(storage, table.name.as_str())
        .await?
        .into_iter()
        .map(|name| ComplexColumnName {
            table: table.clone(),
            name: name.value,
        })
        .collect::<Vec<ComplexColumnName>>())
}
