use {
	crate::{ExecuteError, Glue, Result, Value},
	sqlparser::ast::{Ident, SetVariableValue},
};

impl Glue {
	pub async fn set_variable(
		&mut self,
		variable: &Ident,
		value: &[SetVariableValue],
	) -> Result<()> {
		let first_value = value.get(0).ok_or(ExecuteError::MissingComponentsForSet)?;
		let value: Value = match first_value {
			SetVariableValue::Ident(..) => unimplemented!(),
			SetVariableValue::Literal(literal) => literal.try_into()?,
		};
		let name = variable.value.clone();
		self.get_mut_context()?.set_variable(name, value);
		Ok(())
	}
}
