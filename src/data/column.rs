use {
	crate::ValueType,
	serde::{Deserialize, Serialize},
	sqlparser::{
		ast::{ColumnDef, ColumnOption, ColumnOptionDef, Expr, Ident},
		dialect::keywords::Keyword,
		tokenizer::{Token, Word},
	},
	std::fmt::Debug,
};

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct Column {
	pub name: String,
	pub data_type: ValueType,
	pub default: Option<ValueDefault>,

	pub is_nullable: bool,
	pub is_unique: bool,
}

impl From<&ColumnDef> for Column {
	fn from(column_def: &ColumnDef) -> Self {
		column_def.clone().into()
	}
}
impl From<ColumnDef> for Column {
	fn from(column_def: ColumnDef) -> Self {
		let ColumnDef {
			name: Ident { value: name, .. },
			data_type,
			options,
			..
		} = column_def;

		let is_nullable = options
			.iter()
			.any(|ColumnOptionDef { option, .. }| matches!(option, ColumnOption::Null));

		let is_unique = options
			.iter()
			.any(|ColumnOptionDef { option, .. }| matches!(option, ColumnOption::Unique { .. }));

		let default = options
			.iter()
			.find_map(|ColumnOptionDef { option, .. }| match option {
				ColumnOption::Default(expr) => Some(ValueDefault::Recipe(expr.clone())),
				ColumnOption::DialectSpecific(tokens)
					if matches!(
						tokens[..],
						[
							Token::Word(Word {
								keyword: Keyword::AUTO_INCREMENT,
								..
							}),
							..
						]
					) =>
				{
					Some(ValueDefault::AutoIncrement(1))
				}
				_ => None,
			});

		Self {
			name,
			data_type: data_type.into(),
			default,
			is_nullable,
			is_unique,
		}
	}
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum ValueDefault {
	Recipe(Expr), // TODO: Recipe serialisation
	AutoIncrement(u64),
}
