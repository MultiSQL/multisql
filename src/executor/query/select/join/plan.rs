use {
	super::{JoinManual, JoinType},
	crate::{
		executor::{
			fetch::fetch_columns,
			types::{ColumnInfo, ComplexTableName},
			MetaRecipe,
		},
		Context, JoinError, Result, StorageInner, Glue
	},
	std::cmp::Ordering,
};

#[derive(Debug)]
pub struct JoinPlan {
	pub database: String,
	pub table: String,
	pub columns: Vec<ColumnInfo>,
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
		glue: &Glue,
	) -> Result<Self> {
		let JoinManual {
			table,
			constraint,
			join_type,
		} = join_manual;
		let columns = get_columns(glue, table.clone()).await?;
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
	pub fn calculate_needed_tables(&mut self, table_columns: &[Vec<ColumnInfo>]) {
		self.needed_tables = table_columns
			.iter()
			.enumerate()
			.filter_map(|(index, columns)| {
				if columns.iter().any(|table_column| {
					self.constraint
						.meta
						.objects
						.iter()
						.any(|constraint_column| {
							constraint_column
								.as_ref()
								.map(|constraint_column| table_column == constraint_column)
								.unwrap_or(false)
						})
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
	glue: &Glue,
	table: ComplexTableName,
) -> Result<Vec<ColumnInfo>> {
	if let Some((context_table_labels, ..)) = glue.get_context().tables.get(&table.name) {
		Ok(context_table_labels
			.iter()
			.map(|name| ColumnInfo {
				table: table.clone(),
				name: name.clone(),
				index: None,
			})
			.collect::<Vec<ColumnInfo>>())
	} else {
		let storage = glue.get_storages()
			.iter()
			.find_map(|(name, storage)| {
				if name == &table.database {
					Some(&**storage)
				} else {
					None
				}
			})
			.or_else(|| storages.get(0).map(|(_, storage)| &**storage))
			.ok_or_else(|| JoinError::TableNotFound(table.clone()))?;

		fetch_columns(storage, table).await
	}
}
