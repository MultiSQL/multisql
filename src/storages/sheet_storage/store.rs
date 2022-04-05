use {
	crate::{Result, Row, RowIter, Schema, Store, Value, SheetStorage},
	async_trait::async_trait,
};

#[async_trait(?Send)]
impl Store for SheetStorage {
	async fn fetch_schema(&self, sheet: &str) -> Result<Option<Schema>> {
		let mut sheet_range = self.workbook().worksheet_range(sheet);
		let header: Vec<String> = sheet_range.rows().next().to_vec();
		let column_defs = headers
		.into_iter()
		.map(|header| ColumnDef {
			name: Ident {
				value: header,
				quote_style: None,
			},
			data_type: DataType::Text,
			collation: None,
			options: vec![],
		})
		.collect();

		Ok(Some(Schema {
			table_name: sheet.to_string(),
			column_defs,
			indexes: vec![],
		}))
	}
}
