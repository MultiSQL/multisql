use {
	super::{columns_to_positions, validate},
	crate::{data::Schema, ComplexTableName, ExecuteError, Glue, Payload, Result, Row},
	sqlparser::ast::{Ident, ObjectName, Query},
};

impl Glue {
	pub async fn insert(
		&mut self,
		table_name: &ObjectName,
		columns: &[Ident],
		source: &Query,
		expect_data: bool,
	) -> Result<Payload> {
		let ComplexTableName {
			name: table_name,
			database,
			..
		} = &table_name.try_into()?;
		let Schema {
			column_defs,
			indexes,
			..
		} = self
			.get_database(database)?
			.fetch_schema(table_name)
			.await?
			.ok_or(ExecuteError::TableNotExists)?;

		// TODO: Multi storage
		let (labels, mut rows) = self.query(source.clone()).await?;
		let column_positions = columns_to_positions(&column_defs, columns)?;

		validate(&column_defs, &column_positions, &mut rows)?;
		let mut rows: Vec<Row> = rows.into_iter().map(Row).collect();
		#[cfg(feature = "auto-increment")]
		self.auto_increment(database, table_name, &column_defs, &mut rows)
			.await?;
		self.validate_unique(database, table_name, &column_defs, &rows, None)
			.await?;

		let num_rows = rows.len();

		let database = &mut **self.get_mut_database(database)?;

		let result = database.insert_data(table_name, rows.clone()).await;

		let result = result.map(|_| {
			if expect_data {
				Payload::Select { labels, rows }
			} else {
				Payload::Insert(num_rows)
			}
		})?;

		for index in indexes.iter() {
			// TODO: Should definitely be just inserting an index record
			index.reset(database, table_name, &column_defs).await?; // TODO: Not this; optimise
		}

		Ok(result)
	}
}
