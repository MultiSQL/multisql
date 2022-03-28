use {
	super::AlterError,
	crate::{data::schema::ColumnOptionExt, result::Result},
	sqlparser::ast::{ColumnDef, ColumnOptionDef, DataType},
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

	#[cfg(feature = "auto-increment")]
	if !matches!(data_type, DataType::Int(_))
		&& options
			.iter()
			.any(|ColumnOptionDef { option, .. }| option.is_auto_increment())
	{
		return Err(AlterError::UnsupportedDataTypeForAutoIncrementColumn(
			name.to_string(),
			data_type.to_string(),
		)
		.into());
	}

	Ok(())
}
