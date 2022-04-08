use {sqlparser::ast::{TableFactor, ObjectName as AstObjectName}, crate::{Value, JoinError, Result}, serde::Serialize, std::fmt::Debug};

pub type Alias = Option<String>;
pub type Label = String;
pub type Row = Vec<Value>;
pub type LabelsAndRows = (Vec<Label>, Vec<Row>);
pub type ObjectName = Vec<String>;

#[derive(Debug, Clone)]
pub struct ColumnInfo {
	pub table: ComplexTableName,
	pub name: String,
	pub index: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ComplexTableName {
	pub database: Option<String>,
	pub alias: Alias,
	pub name: String,
}
impl TryFrom<&AstObjectName> for ComplexTableName {
	type Error = crate::Error;
	fn try_from (name: &AstObjectName) -> Result<Self> {

			let name_parts = name.0.len();
			if !(1..=2).contains(&name_parts) {
				return Err(JoinError::UnimplementedNumberOfComponents.into());
			}
			let database = if name_parts == 2 {
				Some(name.0.get(0).unwrap().value.clone())
			} else {
				None
			};
			let name = name.0.last().unwrap().value.clone();
			Ok(Self {
				database,
				name,
				alias: None,
			})
	}
}
impl TryFrom<TableFactor> for ComplexTableName {
	type Error = crate::Error;
	fn try_from (table: TableFactor) -> Result<Self> {
		match table {
			TableFactor::Table { name, alias, .. } => {
				let name_parts = name.0.len();
				if !(1..=2).contains(&name_parts) {
					return Err(JoinError::UnimplementedNumberOfComponents.into());
				}
				let database = if name_parts == 2 {
					Some(name.0.get(0).unwrap().value.clone())
				} else {
					None
				};
				let name = name.0.last().unwrap().value.clone();
				let alias = alias.map(|alias| alias.name.value);
				Ok(Self {
					database,
					name,
					alias,
				})
			}
			_ => Err(JoinError::UnimplementedTableType.into()),
		}
	}
}


impl ColumnInfo {
	pub fn of_name(name: String) -> Self {
		ColumnInfo {
			table: ComplexTableName {
				database: None,
				name: String::new(),
				alias: None,
			},
			name,
			index: None,
		}
	}
}

impl PartialEq<ObjectName> for ColumnInfo {
	fn eq(&self, other: &ObjectName) -> bool {
		let mut other = other.clone();
		other.reverse();
		let names_eq = other
			.get(0)
			.map(|column| column == &self.name)
			.unwrap_or(false);
		let tables_eq = other
			.get(1)
			.map(|table| {
				table == &self.table.name
					|| self
						.table
						.alias
						.as_ref()
						.map(|alias| table == alias)
						.unwrap_or(false)
			})
			.unwrap_or(true);
		names_eq && tables_eq
	}
}
