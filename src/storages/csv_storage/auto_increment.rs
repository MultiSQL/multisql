use {
    super::CSVStorage,
    crate::{AutoIncrement, Result, WIPError},
    async_trait::async_trait,
    linecount::count_lines,
};

#[async_trait(?Send)]
impl AutoIncrement for CSVStorage {
    async fn generate_increment_values(
        &mut self,
        _table_name: String,
        columns: Vec<(
            usize,  /*index*/
            String, /*name*/
            i64,    /*row_count*/
        ) /*column*/>,
    ) -> Result<
        Vec<(
            /*column*/ (usize /*index*/, String /*name*/),
            /*start_value*/ i64,
        )>,
    > {
        let lines: i64 = count_lines(
            std::fs::File::open(self.path.as_str())
                .map_err(|error| WIPError::Debug(format!("{:?}", error)))?,
        )
        .map_err(|error| WIPError::Debug(format!("{:?}", error)))? as i64;
        Ok(columns
            .into_iter()
            .map(|(index, name, _)| ((index, name), lines))
            .collect())
    }

    async fn set_increment_value(
        &mut self,
        _table_name: &str,
        _column_name: &str,
        _end: i64,
    ) -> Result<()> {
        Ok(())
    }
}
