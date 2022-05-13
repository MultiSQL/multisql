use {
	crate::{Column, ComplexTableName, Glue, Result, Row, Schema, Value, ValueType},
	lazy_static::lazy_static,
	sqlparser::ast::{ObjectName, Query},
};

impl Glue {
	pub async fn create_view(
		&mut self,
		name: &ObjectName,
		query: &Box<Query>,
		or_replace: bool,
	) -> Result<()> {
		let ComplexTableName { name, database, .. } = &name.try_into()?;
		// TODO: Parse then serialize as SQL (#140)
		// TODO: Handle
		let query = serde_yaml::to_string(&*query).unwrap();

		// Make view table if not yet exists
		self.add_table(database.clone(), VIEW_TABLE.clone(), true)
			.await?;
		self.insert_data(
			database,
			VIEW_TABLE_NAME,
			vec![Row(vec![Value::Str(name.clone()), Value::Str(query)])],
		)
		.await
		.or_else(|err| {
			if or_replace {
				Ok(()) // TODO: Update
			} else {
				Err(err)
			}
		})
	}
}
const VIEW_TABLE_NAME: &str = "_view";

lazy_static! {
	static ref VIEW_TABLE: Schema = Schema {
		table_name: String::from(VIEW_TABLE_NAME),
		column_defs: vec![
			Column {
				name: String::from("name"),
				data_type: ValueType::Str,
				default: None,
				is_nullable: false,
				is_unique: true,
			},
			Column {
				name: String::from("query"),
				data_type: ValueType::Str,
				default: None,
				is_nullable: false,
				is_unique: false,
			},
		],
		indexes: vec![],
	};
}
