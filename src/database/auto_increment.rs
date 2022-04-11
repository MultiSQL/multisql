use {
	crate::{DatabaseError, Result},
	async_trait::async_trait,
};

#[async_trait(?Send)]
pub trait AutoIncrement {
	async fn generate_increment_values(
		&mut self,
		_table_name: String,
		_columns: Vec<(
			usize,  /*index*/
			String, /*name*/
			i64,    /*row_count*/
		) /*column*/>, // TODO: Use struct
	) -> Result<
		Vec<(
			/*column*/ (usize /*index*/, String /*name*/),
			/*start_value*/ i64,
		)>,
	> {
		Err(DatabaseError::Unimplemented.into())
	}

	async fn set_increment_value(
		&mut self,
		_table_name: &str,
		_column_name: &str,
		_end: i64,
	) -> Result<()> {
		Err(DatabaseError::Unimplemented.into())
	}
}
