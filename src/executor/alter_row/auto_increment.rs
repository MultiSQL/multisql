#![cfg(feature = "auto-increment")]
use crate::{Column, Result, Row, StorageInner, Value, ValueDefault};

pub async fn auto_increment(
	storage: &mut StorageInner,
	table_name: &str,
	columns: &[Column],
	rows: &mut [Row],
) -> Result<()> {
	let auto_increment_columns = columns
		.iter()
		.enumerate()
		.filter(|(_, column)| matches!(column.default, Some(ValueDefault::AutoIncrement(_))))
		.map(|(index, column)| {
			(
				index,
				column.name.clone(),
				rows.iter()
					.filter(|row| matches!(row.0.get(index), Some(Value::Null)))
					.count() as i64,
			)
		})
		.collect();

	let column_values = storage
		.generate_increment_values(table_name.to_string(), auto_increment_columns)
		.await?;

	let mut column_values = column_values;
	for row in rows.into_iter() {
		for ((index, _name), value) in &mut column_values {
			let cell = row.0.get_mut(*index).unwrap();
			if matches!(cell, Value::Null) {
				*cell = Value::I64(*value);
				*value += 1;
			}
		}
	}
	Ok(())
}
