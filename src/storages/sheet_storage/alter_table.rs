use {
	crate::{AlterTable, Result, SheetStorage},
	async_trait::async_trait,
};

#[async_trait(?Send)]
impl AlterTable for SheetStorage {
	async fn rename_schema(&mut self, sheet_name: &str, new_sheet_name: &str) -> Result<()> {
		self.get_sheet_mut(sheet_name)?.set_title(new_sheet_name);
		Ok(())
	}
}
