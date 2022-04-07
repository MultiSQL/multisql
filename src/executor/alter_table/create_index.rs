use {
	crate::{data::get_name, AlterError, ExecuteError, Index, Result, StorageInner},
	sqlparser::ast::{Expr, ObjectName, OrderByExpr},
};

pub async fn create_index(
	storage: &mut StorageInner,
	table: &ObjectName,
	name: &ObjectName,
	columns: &[OrderByExpr],
	unique: bool,
	if_not_exists: bool,
) -> Result<()> {
	let name = name
		.0
		.last()
		.ok_or(ExecuteError::QueryNotSupported)?
		.value
		.clone();

	let table_name = get_name(table)?;

	let schema = storage
		.fetch_schema(table_name)
		.await?
		.ok_or(ExecuteError::TableNotExists)?;

	if schema.indexes.iter().any(|index| index.name == name) {
		if !if_not_exists {
			Err(AlterError::AlreadyExists(name).into())
		} else {
			Ok(())
		}
	} else {
		let mut columns = columns.iter();
		let column = columns.next().and_then(|column| match column.expr.clone() {
			Expr::Identifier(ident) => Some(ident.value),
			_ => None,
		});
		if columns.next().is_some() {
			Err(AlterError::UnsupportedNumberOfIndexColumns(name).into())
		} else if column
			.as_ref()
			.and_then(|column| {
				schema
					.column_defs
					.iter()
					.find(|column_def| &column_def.name == column)
			})
			.is_none()
		{
			Err(AlterError::ColumnNotFound(
				table_name.clone(),
				column.unwrap_or_else(|| String::from("NILL")),
			)
			.into())
		} else if let Some(column) = column {
			let mut schema = schema.clone();
			let index = Index::new(name, column, unique);
			index
				.reset(storage, table_name, &schema.column_defs)
				.await?;
			schema.indexes.push(index);
			storage.replace_schema(table_name, schema).await
		} else {
			unreachable!()
		}
	}
}
