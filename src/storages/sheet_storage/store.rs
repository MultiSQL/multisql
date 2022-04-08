use crate::StorageError;
use {
	crate::{
		Cast, Column, Result, Row, RowIter, Schema, SheetStorage, SheetStorageError, Store, Value,
	},
	async_trait::async_trait,
	std::convert::TryFrom,
	umya_spreadsheet::{Cell, Worksheet},
};

#[async_trait(?Send)]
impl Store for SheetStorage {
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
	async fn scan_data(&self, sheet_name: &str) -> Result<RowIter> {
		let sheet = self.book.get_sheet_by_name(sheet_name).unwrap();
		let Schema { column_defs, .. } = schema_from_sheet(sheet)?;

		let rows: Vec<Result<(Value, Row)>> = sheet
			.get_row_dimensions()
			.into_iter()
			.filter_map(|row| {
				let key = row.get_row_num();
				if key == &1 {
					return None; // Header
				}
				Some(
					sheet
						.get_collection_by_row(key)
						.into_iter()
						.zip(&column_defs)
						.map(|((_, cell), Column { data_type, .. })| {
							Ok(Value::Str(String::from(cell.get_value()))
								.cast_valuetype(data_type)
								.unwrap_or(Value::Null))
						})
						.collect::<Result<Vec<_>>>()
						.map(|row| (Value::I64((*key).into()), Row(row))),
				)
			})
			.collect();
		Ok(Box::new(rows.into_iter()))
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
			_ => return Err(StorageError::Unimplemented.into()),
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
		table_name: sheet.get_title().to_string(),
		column_defs,
		indexes: vec![],
	})
}
