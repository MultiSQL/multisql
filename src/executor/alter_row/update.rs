use {
	super::{auto_increment, columns_to_positions, validate, validate_unique},
	crate::{
		data::{get_name, Schema},
		executor::types::{ComplexColumnName, Row as VecRow},
		Context, ExecuteError, MetaRecipe, Payload, PlannedRecipe, RecipeUtilities, Result, Row,
		StorageInner, Value,
	},
	sqlparser::ast::{Assignment, ColumnDef, Expr, TableFactor, TableWithJoins},
};

pub async fn update(
	storage: &mut StorageInner,
	context: &Context,
	table: &TableWithJoins,
	selection: &Option<Expr>,
	assignments: &Vec<Assignment>,
) -> Result<Payload> {
	// TODO: Handle tables properly
	// TEMP: Just grab first table
	let table = table
		.joins
		.get(0)
		.ok_or(ExecuteError::QueryNotSupported.into())
		.and_then(|table| match &table.relation {
			TableFactor::Table { name, .. } => get_name(&name).map(|name| name.clone()),
			_ => Err(ExecuteError::QueryNotSupported.into()),
		})?;
	let Schema { column_defs, .. } = storage
		.fetch_schema(&table)
		.await?
		.ok_or(ExecuteError::TableNotExists)?;

	let columns = column_defs
		.clone()
		.into_iter()
		.map(|column_def| {
			let ColumnDef { name, .. } = column_def;
			ComplexColumnName::of_name(name.value)
		})
		.collect();

	let filter = selection
		.clone()
		.map(|selection| {
			PlannedRecipe::new(
				MetaRecipe::new(selection)?.simplify_by_context(context)?,
				&columns,
			)
		})
		.unwrap_or(Ok(PlannedRecipe::TRUE))?;

	let assignments = assignments
		.into_iter()
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
				MetaRecipe::new(value.clone())?.simplify_by_context(context)?,
				&columns,
			)?;
			Ok((index, recipe))
		})
		.collect::<Result<Vec<(usize, PlannedRecipe)>>>()?;

	let keyed_rows = storage
		.scan_data(&table)
		.await?
		.into_iter()
		.filter_map(|row_result| {
			let (key, row) = match row_result {
				Ok(keyed_row) => keyed_row,
				Err(error) => return Some(Err(error)),
			};

			let row = row.0;

			let confirm_constraint = filter.confirm_constraint(&row.clone());
			if let Ok(false) = confirm_constraint {
				return None;
			} else if let Err(error) = confirm_constraint {
				return Some(Err(error));
			}
			let row = row
				.iter()
				.enumerate()
				.map(|(index, old_value)| {
					assignments
						.iter()
						.find(|(assignment_index, _)| assignment_index == &index)
						.map(|(_, assignment_recipe)| {
							assignment_recipe.clone().simplify_by_row(&row)?.confirm()
						})
						.unwrap_or(Ok(old_value.clone()))
				})
				.collect::<Result<VecRow>>();
			Some(row.map(|row| (key, row)))
		})
		.collect::<Result<Vec<(Value, VecRow)>>>()?;

	let column_positions = columns_to_positions(&column_defs, &[])?;
	let (keys, rows): (Vec<Value>, Vec<VecRow>) = keyed_rows.into_iter().unzip();
	let rows = validate(&column_defs, &column_positions, rows)?;

	let table = table.as_str();
	#[cfg(feature = "auto-increment")]
	let rows = auto_increment(&mut *storage, table, &column_defs, rows).await?;
	validate_unique(&*storage, table, &column_defs, &rows, Some(&keys)).await?;
	let keyed_rows: Vec<(Value, Row)> = keys.into_iter().zip(rows.into_iter().map(Row)).collect();
	let num_rows = keyed_rows.len();
	storage
		.update_data(keyed_rows)
		.await
		.map(|_| Payload::Update(num_rows))
}
