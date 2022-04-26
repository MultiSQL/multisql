use crate::DatabaseError;
use {
	crate::{Cast, Column, DBBase, Plane, Result, Row, Schema, SheetDatabase, Value},
	async_trait::async_trait,
	std::convert::TryFrom,
	umya_spreadsheet::{Cell, Worksheet},
};

#[async_trait(?Send)]
impl DBBase for SheetDatabase {
	async fn fetch_schema(&self, sheet_name: &str) -> Result<Option<Schema>> {
		if let Ok(sheet) = self.book.get_sheet_by_name(sheet_name) {
			schema_from_sheet(sheet).map(Some)
		} else {
			Ok(None)
		}
	}
	async fn scan_schemas(&self) -> Result<Vec<Schema>> {
		self.book
			.get_sheet_collection()
			.iter()
			.map(schema_from_sheet)
			.collect()
	}
	async fn scan_data(&self, sheet_name: &str) -> Result<Plane> {
		let sheet = self.book.get_sheet_by_name(sheet_name).unwrap();
		let Schema { column_defs, .. } = schema_from_sheet(sheet)?;

		let row_count = sheet.get_highest_row();
		let col_count = sheet.get_highest_column();

		let rows = vec![vec![None; col_count as usize]; (row_count as usize) - 1];
		let rows = sheet
			.get_collection_to_hashmap()
			.iter()
			.filter(|((row, _col), _)| row != &1)
			.fold(rows, |mut rows, ((row_num, col_num), cell)| {
				rows[(row_num - 2) as usize][(col_num - 1) as usize] = Some(cell.clone());
				rows
			});

		rows.into_iter()
			.enumerate()
			.map(|(pk, row)| {
				(
					Value::U64((pk + 2) as u64),
					Row(row
						.into_iter()
						.zip(&column_defs)
						.map(|(cell, Column { data_type, .. })| {
							Value::Str(
								cell.map(|cell| cell.get_value().to_string())
									.unwrap_or_default(),
							)
							.cast_valuetype(data_type)
							.unwrap_or(Value::Null)
						})
						.collect()),
				)
			})
			.map(Ok)
			.collect::<Result<Vec<(Value, Row)>>>()
	}
}

impl TryFrom<Cell> for Value {
	type Error = crate::Error;
	fn try_from(cell: Cell) -> Result<Self> {
		Ok(match cell.get_data_type() {
			Cell::TYPE_STRING | Cell::TYPE_STRING2 => Value::Str(cell.get_value().to_string()),
			Cell::TYPE_BOOL => Value::Bool(Value::Str(cell.get_value().to_string()).cast()?),
			Cell::TYPE_NUMERIC => Value::F64(Value::Str(cell.get_value().to_string()).cast()?),
			Cell::TYPE_NULL => Value::Null,
			_ => return Err(DatabaseError::Unimplemented.into()),
		})
	}
}

fn schema_from_sheet(sheet: &Worksheet) -> Result<Schema> {
	let mut column_defs: Vec<(_, Column)> = sheet
		.get_comments()
		.iter()
		.filter_map(|comment| {
			let coordinate = comment.get_coordinate();
			if coordinate.get_row_num() == &1 {
				let col = coordinate.get_col_num();
				let text = comment.get_text().get_text();
				let column_def: Column = serde_yaml::from_str(text).unwrap_or_default();
				Some(Ok((col, column_def)))
			} else {
				None
			}
		})
		.collect::<Result<Vec<_>>>()?;
	column_defs.sort_by(|(col_a, _), (col_b, _)| col_a.cmp(col_b));
	let column_defs = column_defs
		.into_iter()
		.map(|(_, column_def)| column_def)
		.collect();

	Ok(Schema {
		table_name: sheet.get_name().to_string(),
		column_defs,
		indexes: vec![],
	})
}
