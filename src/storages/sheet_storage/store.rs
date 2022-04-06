use crate::StorageError;

use {
	crate::{Cast, Result, Row, RowIter, Schema, SheetStorage, Store, Value},
	async_trait::async_trait,
	sqlparser::ast::{ColumnDef, ColumnOption, ColumnOptionDef, DataType, Ident},
	std::convert::TryFrom,
	umya_spreadsheet::{Cell, CellValue, Worksheet},
};

#[async_trait(?Send)]
impl Store for SheetStorage {
	async fn fetch_schema(&self, sheet_name: &str) -> Result<Option<Schema>> {
		Ok(if let Ok(sheet) = self.book.get_sheet_by_name(sheet_name) {
			Some(schema_from_sheet(sheet))
		} else {
			None
		})
	}
	async fn scan_schemas(&self) -> Result<Vec<Schema>> {
		Ok(self
			.book
			.get_sheet_collection()
			.iter()
			.map(schema_from_sheet)
			.collect())
	}
	async fn scan_data(&self, sheet_name: &str) -> Result<RowIter> {
		let sheet = self.book.get_sheet_by_name(sheet_name).unwrap();
		let Schema { column_defs, .. } = schema_from_sheet(&sheet);

		// Skip header
		let rows: Vec<Result<(Value, Row)>> = sheet
			.get_row_dimensions()
			.into_iter()
			//.skip(1)
			.filter_map(|row| {
				let key = row.get_row_num();
				if key == &1 {
					return None;
				}
				Some(
					sheet
						.get_collection_by_row(key)
						.into_iter()
						.zip(&column_defs)
						.map(|((_, cell), ColumnDef { data_type, .. })| {
							Ok(Value::Str(String::from(cell.get_value()))
								.cast_datatype(data_type)
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

fn schema_from_sheet(sheet: &Worksheet) -> Schema {
	let mut column_defs: Vec<(_, ColumnDef)> = sheet
		.get_comments()
		.iter()
		.filter_map(|comment| {
			let coordinate = comment.get_coordinate();
			if coordinate.get_row_num() == &1 {
				let col = coordinate.get_col_num();
				let text = comment.get_text().get_text();
				let column_def: ColumnDef = serde_yaml::from_str(&text).unwrap();
				Some((col, column_def))
			} else {
				None
			}
		})
		.collect();
	column_defs.sort_by(|(col_a, _), (col_b, _)| col_a.cmp(col_b));
	let column_defs = column_defs
		.into_iter()
		.map(|(_, column_def)| column_def)
		.collect();

	Schema {
		table_name: sheet.get_title().to_string(),
		column_defs,
		indexes: vec![],
	}
}
