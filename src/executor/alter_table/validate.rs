use {
	crate::{Result, AlterError},
	sqlparser::ast::{ColumnDef, DataType},
};

pub fn validate(column_def: &ColumnDef) -> Result<()> {
	let ColumnDef {
		data_type,
		options,
		name,
		..
	} = column_def;

	if !matches!(
		data_type,
		DataType::Boolean | DataType::Int(_) | DataType::Float(_) | DataType::Text
	) {
		return Err(AlterError::UnsupportedDataType(data_type.to_string()).into());
	}

	Ok(())
}
