use {
	crate::{
		data::get_name, AlterError, CreateError, Error, ExecuteError, Glue, Index, Result,
		SchemaDiff,
	},
	sqlparser::ast::{Expr, ObjectName, OrderByExpr},
};

impl Glue {
	pub async fn create_index(
		&mut self,
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
		let database = &mut **self.get_mut_database(&None)?;

		let schema = database
			.fetch_schema(table_name)
			.await?
			.ok_or(ExecuteError::TableNotExists)?;

		if schema.indexes.iter().any(|index| index.name == name) {
			if !if_not_exists {
				Err(Error::Create(CreateError::AlreadyExists(name).into()))
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
				let schema = schema.clone();
				let index = Index::new(name, column, unique);
				index
					.reset(database, table_name, &schema.column_defs)
					.await?;
				database
					.alter_table(table_name, SchemaDiff::new_add_index(index))
					.await
			} else {
				unreachable!()
			}
		}
	}
}
