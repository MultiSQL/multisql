use umya_spreadsheet::{Border, Comment, PatternValues, RichText, Style, TextElement};
use {
	crate::{
		Cast, DBMut, DatabaseError, Result, Row, Schema, SchemaChange, SchemaDiff, SheetDatabase,
		SheetDatabaseError, Value,
	},
	async_trait::async_trait,
};

#[async_trait(?Send)]
impl DBMut for SheetDatabase {
	async fn insert_schema(&mut self, schema: &Schema) -> Result<()> {
		let mut style = Style::default();
		style
			.get_fill_mut()
			.get_pattern_fill_mut()
			.set_pattern_type(PatternValues::Gray125);
		style
			.get_borders_mut()
			.get_bottom_mut()
			.set_border_style(Border::BORDER_MEDIUM);
		style
			.get_borders_mut()
			.get_left_mut()
			.set_border_style(Border::BORDER_THIN);
		style
			.get_borders_mut()
			.get_right_mut()
			.set_border_style(Border::BORDER_THIN);

		let Schema {
			column_defs,
			table_name: sheet_name,
			..
		} = schema;
		let sheet = self
			.book
			.new_sheet(sheet_name)
			.map_err(|_| SheetDatabaseError::FailedToCreateSheet)?;
		column_defs
			.iter()
			.enumerate()
			.try_for_each::<_, Result<_>>(|(index, column_def)| {
				let col = (index as u32) + 1;
				let row = 1;
				sheet
					.get_cell_by_column_and_row_mut(&col, &row)
					.set_value(&column_def.name)
					.set_style(style.clone());
				let mut comment_text_element = TextElement::default();
				comment_text_element.set_text(
					serde_yaml::to_string(&column_def)
						.map_err(|_| SheetDatabaseError::FailedColumnParse)?,
				);
				let mut comment_text = RichText::default();
				comment_text.add_rich_text_elements(comment_text_element);
				let mut comment = Comment::default();
				comment
					.set_text(comment_text)
					.get_coordinate_mut()
					.set_col_num(col)
					.set_row_num(row);
				sheet.add_comments(comment);
				Ok(())
			})?;
		self.save()
	}
	async fn insert_data(&mut self, sheet_name: &str, rows: Vec<Row>) -> Result<()> {
		let sheet = self.get_sheet_mut(sheet_name)?;
		let row_init = sheet.get_row_dimensions().len() + 1; // TODO: Not this
		rows.into_iter()
			.enumerate()
			.for_each(|(row_num, Row(row))| {
				row.into_iter().enumerate().for_each(|(col_num, cell)| {
					sheet
						.get_cell_by_column_and_row_mut(
							&(col_num as u32 + 1),
							&((row_num + row_init) as u32),
						)
						.set_value(cell);
				})
			});
		self.save()
	}

	async fn delete_schema(&mut self, sheet_name: &str) -> Result<()> {
		self.book
			.remove_sheet_by_name(sheet_name)
			.map_err(|_| SheetDatabaseError::FailedToGetSheet)?;
		self.save()
	}

	async fn update_data(&mut self, sheet_name: &str, rows: Vec<(Value, Row)>) -> Result<()> {
		let sheet = self.get_sheet_mut(sheet_name)?;
		rows.into_iter()
			.try_for_each::<_, Result<()>>(|(key, Row(row))| {
				let row_num: i64 = key.cast()?;
				row.into_iter().enumerate().for_each(|(col_num, cell)| {
					sheet
						.get_cell_by_column_and_row_mut(&(col_num as u32 + 1), &(row_num as u32))
						.set_value(cell);
				});
				Ok(())
			})?;
		self.save()
	}

	async fn delete_data(&mut self, sheet_name: &str, rows: Vec<Value>) -> Result<()> {
		let sheet = self.get_sheet_mut(sheet_name)?;
		rows.into_iter().try_for_each::<_, Result<()>>(|key| {
			let row_num: u64 = key.cast()?;
			sheet.remove_row(&(row_num as u32), &1);
			Ok(())
		})?;
		self.save()
	}

	async fn alter_table(&mut self, sheet_name: &str, schema_diff: SchemaDiff) -> Result<()> {
		let changes = schema_diff.get_changes();
		let sheet = self.get_sheet_mut(sheet_name)?;
		for change in changes.into_iter() {
			use SchemaChange::*;
			match change {
				RenameTable(new_name) => {
					sheet.set_name(new_name);
				}
				_ => return Err(DatabaseError::Unimplemented.into()),
				// TODO
			};
		}

		self.save()
	}
}
