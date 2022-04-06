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

		// Skip header
		let rows: Vec<Result<(Value, Row)>> = sheet
			.get_row_dimensions()
			.into_iter()
			.skip(1)
			.map(|row| {
				let key = row.get_row_num();
				sheet
					.get_collection_by_row(key)
					.into_iter()
					.map(|(_, cell)| cell.clone().try_into())
					.collect::<Result<Vec<_>>>()
					.map(|row| (Value::I64((*key).into()), Row(row)))
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
			Cell::TYPE_ERROR | _ => {
				unimplemented!()
			}
		})
	}
}

fn schema_from_sheet(sheet: &Worksheet) -> Schema {
	let headers = sheet.get_collection_by_row(&1);
	let mut first_row: Vec<&str> = sheet
		.get_collection_by_row(&2)
		.into_iter()
		.map(|(_, fr_cell)| fr_cell.get_data_type())
		.collect();
	first_row.resize_with(headers.len(), || "");

	let column_defs = headers
		.into_iter()
		.zip(first_row)
		.map(|((_, header), fr_cell)| ColumnDef {
			name: Ident {
				value: header.get_value().to_string(),
				quote_style: None,
			},
			data_type: match fr_cell {
				CellValue::TYPE_BOOL => DataType::Boolean,
				CellValue::TYPE_NUMERIC => DataType::Float(None),
				_ => DataType::Text,
			},
			collation: None,
			options: vec![ColumnOptionDef {
				option: ColumnOption::Null,
				name: None,
			}],
		})
		.collect();

	Schema {
		table_name: sheet.get_title().to_string(),
		column_defs,
		indexes: vec![],
	}
}
