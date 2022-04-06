use umya_spreadsheet::{Comment, RichText, TextElement};

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
				column_def,
			)| {
				let col = (index as u32) + 1;
				let row = 1;
				sheet
					.get_cell_by_column_and_row_mut(col, row)
					.set_value(column_def.name.value.clone());
				let mut comment_text_element = TextElement::default();
				comment_text_element.set_text(serde_yaml::to_string(&column_def).unwrap());
				let mut comment_text = RichText::default();
				comment_text.add_rich_text_elements(comment_text_element);
				let mut comment = Comment::default();
				comment.set_text(comment_text).get_coordinate_mut().set_col_num(col).set_row_num(row);
				sheet.add_comments(comment);
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
