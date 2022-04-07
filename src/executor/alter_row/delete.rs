use {
	crate::{
		data::{get_name, Schema},
		executor::types::ColumnInfo,
		Context, ExecuteError, MetaRecipe, Payload, PlannedRecipe, Result, StorageInner, Value,
	},
	sqlparser::ast::{ColumnDef, Expr, ObjectName},
};

pub async fn delete(
	storages: &mut Vec<(String, &mut StorageInner)>,
	context: &mut Context,
	table_name: &ObjectName,
	selection: &Option<Expr>,
) -> Result<Payload> {
	let table_name = get_name(table_name)?;
	let Schema {
		column_defs,
		indexes,
		..
	} = storages[0]
		.1
		.fetch_schema(table_name)
		.await?
		.ok_or(ExecuteError::TableNotExists)?;

	let columns = column_defs
		.clone()
		.into_iter()
		.map(|ColumnDef { name, .. }| ColumnInfo::of_name(name.value))
		.collect::<Vec<ColumnInfo>>();
	let filter = selection
		.clone()
		.map(|selection| {
			PlannedRecipe::new(
				MetaRecipe::new(selection)?.simplify_by_context(context)?,
				&columns,
			)
		})
		.unwrap_or(Ok(PlannedRecipe::TRUE))?;

	let keys = storages[0]
		.1
		.scan_data(table_name)
		.await?
		.filter_map(|row_result| {
			let (key, row) = match row_result {
				Ok(keyed_row) => keyed_row,
				Err(error) => return Some(Err(error)),
			};

			let row = row.0;

			let confirm_constraint = filter.confirm_constraint(&row);
			match confirm_constraint {
				Ok(true) => Some(Ok(key)),
				Ok(false) => None,
				Err(error) => Some(Err(error)),
			}
		})
		.collect::<Result<Vec<Value>>>()?;

	let num_keys = keys.len();

	let result = storages[0]
		.1
		.delete_data(&table_name, keys)
		.await
		.map(|_| Payload::Delete(num_keys))?;

	for index in indexes.iter() {
		index.reset(storages[0].1, table_name, &column_defs).await?; // TODO: Not this; optimise
	}
	Ok(result)
}
