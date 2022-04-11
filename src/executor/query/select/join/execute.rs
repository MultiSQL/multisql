use {
	super::{JoinError, JoinMethod, JoinPlan, JoinType},
	crate::{
		executor::types::{ColumnInfo, Row},
		DatabaseInner, Glue, IndexFilter, Ingredient, MetaRecipe, Method, PlannedRecipe, Recipe,
		Result, Value,
	},
};

#[derive(Debug)]
pub struct JoinExecute {
	pub database: Option<String>,
	pub table: String,
	pub method: JoinMethod,
	pub join_type: JoinType,
	pub widths: (usize, usize),
	pub index_filter: Option<IndexFilter>,
}

impl JoinExecute {
	pub fn new(
		plan: JoinPlan,
		plane_columns: &[ColumnInfo],
		index_filter: Option<IndexFilter>,
	) -> Result<Self> {
		let JoinPlan {
			database,
			table,
			join_type,
			constraint,
			columns,
			..
		} = plan;
		let widths = (plane_columns.len(), columns.len());
		let method = decide_method(constraint, columns, plane_columns)?;
		Ok(Self {
			database,
			table,
			method,
			join_type,
			widths,
			index_filter,
		})
	}
	pub fn set_first_table(&mut self) {
		self.method = JoinMethod::FirstTable;
	}
	pub async fn get_rows<'a>(&self, storage: &DatabaseInner) -> Result<Vec<Row>> {
		if let Some(index_filter) = self.index_filter.clone() {
			storage.scan_data_indexed(self.table.as_str(), index_filter)
		} else {
			storage.scan_data(self.table.as_str())
		}
		.await
		.map(|plane| {
			plane
				.into_iter()
				.map(|(_, row)| row.0)
				.collect::<Vec<Row>>()
		})
	}
	pub async fn execute<'a>(self, glue: &Glue, plane_rows: Vec<Row>) -> Result<Vec<Row>> {
		let rows =
			if let Some((.., context_table_rows)) = glue.get_context()?.tables.get(&self.table) {
				Ok(context_table_rows.clone())
			} else {
				self.get_rows(&**glue.get_database(&self.database)?).await
			}?;
		self.method.run(
			&self.join_type,
			self.widths.0,
			self.widths.1,
			plane_rows,
			rows,
		)
	}
}

fn decide_method(
	constraint: MetaRecipe,
	self_columns: Vec<ColumnInfo>,
	plane_columns: &[ColumnInfo],
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
					.objects
					.get(index_l)
					.ok_or(JoinError::Unreachable)?
					.as_ref()
					.ok_or(JoinError::Unreachable)?;
				let column_r = constraint
					.meta
					.objects
					.get(index_r)
					.ok_or(JoinError::Unreachable)?
					.as_ref()
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
