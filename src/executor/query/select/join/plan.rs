use {
	super::{JoinManual, JoinType},
	crate::{
		recipe::MetaRecipe,
		types::{ColumnInfo, ComplexTableName},
		Glue, Result,
	},
	std::cmp::Ordering,
};

#[derive(Debug)]
pub struct JoinPlan {
	pub database: Option<String>,
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
	pub async fn new<'a>(join_manual: JoinManual, glue: &Glue) -> Result<Self> {
		let JoinManual {
			table,
			constraint,
			join_type,
		} = join_manual;
		let columns = glue.get_columns(table.clone()).await?;
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
