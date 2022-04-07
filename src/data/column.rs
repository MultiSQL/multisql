use {
	crate::Value,
	serde::{Deserialize, Serialize},
	sqlparser::{
		ast::{ColumnDef, ColumnOption, ColumnOptionDef, DataType, Expr, Ident},
		dialect::keywords::Keyword,
		tokenizer::{Token, Word},
	},
};

#[derive(Default, Clone, Serialize, Deserialize)]
struct Column {
	name: String,
	data_type: ValueType,
	default: Option<ValueDefault>,

	is_nullable: bool,
	is_unique: bool,
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

#[derive(Clone, Serialize, Deserialize)]
enum ValueType {
	Bool,
	U64,
	I64,
	F64,
	Str,
	Timestamp,
	Any,
}
impl Default for ValueType {
	fn default() -> Self {
		Self::Any
	}
}
impl From<Value> for ValueType {
	fn from(value: Value) -> Self {
		match value {
			Value::Bool(_) => ValueType::Bool,
			Value::U64(_) => ValueType::U64,
			Value::I64(_) => ValueType::I64,
			Value::F64(_) => ValueType::F64,
			Value::Str(_) => ValueType::Str,
			Value::Timestamp(_) => ValueType::Timestamp,
			_ => ValueType::Any,
		}
	}
}
impl From<DataType> for ValueType {
	fn from(data_type: DataType) -> Self {
		match data_type {
			DataType::Boolean => ValueType::Bool,
			DataType::UnsignedInt(_) => ValueType::U64,
			DataType::Int(_) => ValueType::I64,
			DataType::Float(_) => ValueType::F64,
			DataType::Text => ValueType::Str,
			DataType::Timestamp => ValueType::Timestamp,
			_ => ValueType::Any,
		}
	}
}

#[derive(Clone, Serialize, Deserialize)]
enum ValueDefault {
	Recipe(Expr), // TODO: Recipe serialisation
	AutoIncrement(u64),
}
