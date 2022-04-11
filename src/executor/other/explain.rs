use crate::{ExecuteError, Payload, Row, Schema, DatabaseInner, Value};
use crate::{Glue, Result};
use sqlparser::ast::ObjectName;

impl Glue {
	pub async fn explain(&self, object: &ObjectName) -> Result<Payload> {
		let mut name_vec = object.0.clone();
		let (store_name, opt_table_name) = match name_vec.len() {
			2 => (
				Some(name_vec.remove(0).value),
				Some(name_vec.remove(0).value),
			),
			1 => {
				let name = name_vec.remove(0).value;
				if name == "ALL" {
					let databases: Vec<Row> = self
						.get_database_list()
						.into_iter()
						.map(|name| Row(vec![name.clone().into()]))
						.collect();
					return Ok(Payload::Select {
						labels: vec![String::from("database")],
						rows: databases,
					});
				}
				if name == "ALL_TABLE" {
					let mut tables = vec![];
					for db_name in self.get_database_list().into_iter() {
						tables.extend(
							self.get_database(&Some(db_name.clone()))?
								.get_tables()
								.await?
								.iter()
								.map(|table| Row(vec![db_name.clone().into(), table.clone()])),
						);
					}
					return Ok(Payload::Select {
						labels: vec![String::from("database"), String::from("table")],
						rows: tables,
					});
				} else if self.get_database_list().contains(&&name) {
					(Some(name), None)
				} else {
					(None, Some(name))
				}
			}
			_ => return Err(ExecuteError::ObjectNotRecognised.into()),
		};

		let database = self.get_database(&store_name)?;
		if let Some(table_name) = opt_table_name {
			let Schema { column_defs, .. } = database
				.fetch_schema(&table_name)
				.await?
				.ok_or(ExecuteError::ObjectNotRecognised)?;
			let columns = column_defs
				.iter()
				.map(|column| {
					(
						column.name.clone().into(),
						column.data_type.to_string().into(),
					)
				})
				.map(|(name, data_type)| Row(vec![name, data_type]))
				.collect();
			Ok(Payload::Select {
				labels: vec![String::from("column"), String::from("data_type")],
				rows: columns,
			})
		} else {
			Ok(Payload::Select {
				labels: vec![String::from("table")],
				rows: database
					.get_tables()
					.await?
					.into_iter()
					.map(|table| Row(vec![table]))
					.collect(),
			})
		}
	}
}
impl DatabaseInner {
	async fn get_tables(&self) -> Result<Vec<Value>> {
		Ok(self
			.scan_schemas()
			.await?
			.into_iter()
			.map(|Schema { table_name, .. }| table_name.into())
			.collect())
	}
}
