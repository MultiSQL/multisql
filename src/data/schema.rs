use {
	crate::{Column, Index},
	serde::{Deserialize, Serialize},
};

#[derive(Clone, Serialize, Deserialize)]
pub struct Schema {
	pub table_name: String,
	pub column_defs: Vec<Column>,
	pub indexes: Vec<Index>,
}
