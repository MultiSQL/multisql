use {
	super::{auto_increment, columns_to_positions, validate, validate_unique},
	crate::{
		data::{get_name, Schema},
		executor::query::query,
		Context, ExecuteError, Payload, Result, Row, StorageInner,
	},
	sqlparser::ast::{Ident, ObjectName, Query},
};

pub async fn insert(
	storages: &mut Vec<(String, &mut StorageInner)>,
	context: &mut Context,
	table_name: &ObjectName,
	columns: &[Ident],
	source: &Box<Query>,
	expect_data: bool,
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

	// TODO: Multi storage
	let (labels, rows) = query(storages, context, *source.clone()).await?;
	let column_positions = columns_to_positions(&column_defs, columns)?;

	let rows = validate(&column_defs, &column_positions, rows)?;
	#[cfg(feature = "auto-increment")]
	let rows = auto_increment(storages[0].1, table_name, &column_defs, rows).await?;
	validate_unique(storages[0].1, table_name, &column_defs, &rows, None).await?;
	let rows: Vec<Row> = rows.into_iter().map(Row).collect();

	let num_rows = rows.len();

	let result = storages[0].1.insert_data(table_name, rows.clone()).await;

	let result = result.map(|_| {
		if expect_data {
			Payload::Select { labels, rows }
		} else {
			Payload::Insert(num_rows)
		}
	})?;

	for index in indexes.iter() {
		// TODO: Should definitely be just inserting an index record
		index
			.reset(storages[0].1, table_name, &column_defs)
			.await?; // TODO: Not this; optimise
	}

	Ok(result)
}
