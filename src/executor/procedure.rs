use {
	crate::{ExecuteError, Glue, Payload, Result},
	sqlparser::ast::{Expr, Ident, Value as AstValue},
};

impl Glue {
	pub async fn procedure(&mut self, name: &Ident, parameters: &[Expr]) -> Result<Payload> {
		return match name.value.as_str() {
			"FILE" => {
				if let Some(Ok(query)) = parameters.get(0).map(|path| {
					if let Expr::Value(AstValue::SingleQuotedString(path)) = path {
						std::fs::read_to_string(path).map_err(|_| ())
					} else {
						Err(())
					}
				}) {
					self.execute(&query)
				} else {
					Err(ExecuteError::InvalidFileLocation.into())
				}
			}
			_ => Err(ExecuteError::Unimplemented.into()),
		};
	}
}
