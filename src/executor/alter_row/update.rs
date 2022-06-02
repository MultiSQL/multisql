use {
	super::{columns_to_positions, validate},
	crate::{
		data::Schema,
		recipe::{MetaRecipe, PlannedRecipe, RecipeUtilities},
		types::{ColumnInfo, ComplexTableName, Row as VecRow},
		Column, ExecuteError, Glue, Payload, Result, Row, Value,
	},
	sqlparser::ast::{Assignment, Expr, TableFactor, TableWithJoins},
};

impl Glue {
	pub async fn ast_update(
		&mut self,
		table: &TableWithJoins,
		selection: &Option<Expr>,
		assignments: &[Assignment],
	) -> Result<Payload> {
		// TODO: Complex updates (joins)
		let ComplexTableName {
			name: table,
			database,
			..
		} = match &table.relation {
			TableFactor::Table { name, .. } => name.try_into(),
			_ => Err(ExecuteError::QueryNotSupported.into()),
		}?;
		let Schema {
			column_defs,
			indexes,
			..
		} = self
			.get_database(&database)?
			.fetch_schema(&table)
			.await?
			.ok_or(ExecuteError::TableNotExists)?;

		let columns = column_defs
			.clone()
			.into_iter()
			.map(|Column { name, .. }| ColumnInfo::of_name(name))
			.map(|mut col| {
				col.table.name = table.clone();
				col
			})
			.collect::<Vec<ColumnInfo>>();

		let filter = selection
			.clone()
			.map(|selection| {
				PlannedRecipe::new(
					MetaRecipe::new(selection)?.simplify_by_tempdb(&self.tempdb)?,
					&columns,
				)
			})
			.unwrap_or(Ok(PlannedRecipe::TRUE))?;

		let assignments = assignments
			.iter()
			.map(|assignment| {
				let Assignment { id, value } = assignment;
				let column_compare = id
					.clone()
					.into_iter()
					.map(|component| component.value)
					.collect();
				let index = columns
					.iter()
					.position(|column| column == &column_compare)
					.ok_or(ExecuteError::ColumnNotFound)?;
				let recipe = PlannedRecipe::new(
					MetaRecipe::new(value.clone())?.simplify_by_tempdb(&self.tempdb)?,
					&columns,
				)?;
				Ok((index, recipe))
			})
			.collect::<Result<Vec<(usize, PlannedRecipe)>>>()?;

		let keyed_rows = self
			.get_database(&None)?
			.scan_data(&table)
			.await?
			.into_iter()
			.filter_map(|(key, Row(row))| match filter.confirm_constraint(&row) {
				Ok(false) => None,
				Err(error) => Some(Err(error)),
				Ok(true) => Some(
					row.iter()
						.enumerate()
						.map(|(index, old_value)| {
							assignments
								.iter()
								.find_map(|(assignment_index, assignment_recipe)| {
									if assignment_index == &index {
										Some(
											assignment_recipe
												.clone()
												.simplify_by_row(&row)
												.and_then(|recipe| recipe.confirm()),
										)
									} else {
										None
									}
								})
								.unwrap_or(Ok(old_value.clone()))
						})
						.collect::<Result<VecRow>>()
						.map(|row| (key, row)),
				),
			})
			.collect::<Result<Vec<(Value, VecRow)>>>()?;

		let column_positions = columns_to_positions(&column_defs, &[])?;
		let (keys, mut rows): (Vec<Value>, Vec<VecRow>) = keyed_rows.into_iter().unzip();
		validate(&column_defs, &column_positions, &mut rows)?;

		let table = table.as_str();
		let mut rows: Vec<Row> = rows.into_iter().map(Row).collect();
		#[cfg(feature = "auto-increment")]
		self.auto_increment(&database, table, &column_defs, &mut rows)
			.await?;
		/*self.validate_unique(&database, table, &column_defs, &rows, Some(&keys))
		.await?;*/
		let keyed_rows: Vec<(Value, Row)> = keys.into_iter().zip(rows).collect();
		let num_rows = keyed_rows.len();

		let database = &mut **self.get_mut_database(&database)?;

		let result = database
			.update_data(table, keyed_rows)
			.await
			.map(|_| Payload::Update(num_rows))?;

		for index in indexes.iter() {
			index.reset(database, table, &column_defs).await?; // TODO: Not this; optimise
		}
		Ok(result)
	}
}
