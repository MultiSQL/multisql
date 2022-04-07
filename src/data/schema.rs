use {
	crate::{Column, Index},
	serde::{Deserialize, Serialize},
	sqlparser::ast::{ColumnDef, ColumnOption, ColumnOptionDef, Expr},
};

#[cfg(feature = "auto-increment")]
use sqlparser::{
	dialect::keywords::Keyword,
	tokenizer::{Token, Word},
};

#[derive(Clone, Serialize, Deserialize)]
pub struct Schema {
	pub table_name: String,
	pub column_defs: Vec<Column>,
	pub indexes: Vec<Index>,
}
