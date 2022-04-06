use {
	crate::{Result, Row, Schema, SheetStorage, StoreMut},
	async_trait::async_trait,
	sqlparser::ast::{ColumnDef, DataType},
	umya_spreadsheet::CellValue,
};

#[async_trait(?Send)]
impl StoreMut for SheetStorage {
	async fn insert_schema(&mut self, schema: &Schema) -> Result<()> {
		let Schema {
			column_defs,
			table_name: sheet_name,
			..
		} = schema;
		let sheet = self.book.new_sheet(sheet_name).unwrap();
		column_defs.into_iter().enumerate().for_each(
			|(
				index,
				ColumnDef {
					name, data_type, ..
				},
			)| {
				sheet
					.get_cell_by_column_and_row_mut((index as u32) + 1, 1)
					.set_value(name.value.clone());
				sheet
					.get_cell_by_column_and_row_mut((index as u32) + 1, 2)
					.set_value(match data_type {
						// Ick!
						DataType::Int(_) | DataType::Float(_) => "0",
						DataType::Boolean => "false",
						_ => "",
					})
					.set_data_type(match data_type {
						DataType::Text => CellValue::TYPE_STRING,
						DataType::Int(_) | DataType::Float(_) => CellValue::TYPE_NUMERIC,
						DataType::Boolean => CellValue::TYPE_BOOL,
						_ => CellValue::TYPE_STRING2,
					});
			},
		);
		self.save()
	}
	async fn insert_data(&mut self, sheet_name: &str, rows: Vec<Row>) -> Result<()> {
		let sheet = self.book.get_sheet_by_name_mut(sheet_name).unwrap();
		let mut row_init = sheet.get_row_dimensions().len() + 1; // TODO: Not this
		if row_init == 3 {
			row_init = 2; // TODO: VERY not this
		}
		rows.into_iter()
			.enumerate()
			.for_each(|(row_num, Row(row))| {
				row.into_iter().enumerate().for_each(|(col_num, cell)| {
					sheet
						.get_cell_by_column_and_row_mut(
							col_num as u32 + 1,
							(row_num + row_init) as u32,
						)
						.set_value(cell);
				})
			});
		self.save()
	}
}
