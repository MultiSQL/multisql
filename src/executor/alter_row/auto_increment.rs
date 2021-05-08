#![cfg(feature = "auto-increment")]
use {
	crate::{data::schema::ColumnDefExt, executor::types::Row, Result, StorageInner, Value},
	sqlparser::ast::ColumnDef,
};

pub async fn auto_increment(
	storage: &mut StorageInner,
	table_name: &str,
	column_defs: &[ColumnDef],
	rows: Vec<Row>,
) -> Result<Vec<Row>> {
	let auto_increment_columns = column_defs
		.iter()
		.enumerate()
		.filter(|(_, column_def)| column_def.is_auto_incremented())
		.map(|(index, column_def)| {
			(
				index,
				column_def.name.value.clone(),
				rows.iter()
					.filter(|row| matches!(row.get(index), Some(Value::Null)))
					.count() as i64,
			)
		})
		.collect();

	let column_values = storage
		.generate_increment_values(table_name.to_string(), auto_increment_columns)
		.await?;

	let mut rows = rows;
	let mut column_values = column_values;
	for row in &mut rows {
		for ((index, _name), value) in &mut column_values {
			let cell = row.get_mut(*index).unwrap();
			if matches!(cell, Value::Null) {
				*cell = Value::I64(*value);

				*value += 1;
			}
		}
	}
	Ok(rows)
}
