pub use sqlparser::parser::ParserError;
use sqlparser::{ast::Statement, dialect::GenericDialect, parser::Parser};

pub struct Query(pub Statement);

pub fn parse(sql: &str) -> Result<Vec<Query>, ParserError> {
	let dialect = GenericDialect {};

	Parser::parse_sql(&dialect, sql).map(|parsed| parsed.into_iter().map(Query).collect())
}

pub fn parse_single(sql: &str) -> Result<Query, ParserError> {
	parse(sql)?
		.into_iter()
		.next()
		.ok_or_else(|| ParserError::ParserError(String::from("No Query")))
}
