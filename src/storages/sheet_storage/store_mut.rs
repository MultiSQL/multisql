use {
	crate::{Result, Row, Schema, SheetStorage, StoreMut},
	async_trait::async_trait,
	sqlparser::ast::ColumnDef,
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
		//self.book.insert_new_row(sheet_name, 0, column_defs.len() as u32);
		column_defs
			.into_iter()
			.enumerate()
			.for_each(|(index, ColumnDef { name, .. })| {
				sheet
					.get_cell_by_column_and_row_mut((index as u32)+1, 1)
					.set_value(name.value.clone());
			});
		self.save()
	}
}
