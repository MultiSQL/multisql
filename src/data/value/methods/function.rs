use {
	crate::{Cast, CastWithRules, Convert, Result, Value, ValueError},
	chrono::NaiveDateTime,
	uuid::Uuid,
};

macro_rules! expect_arguments {
	($arguments: expr, $expect: expr) => {
		match $arguments.len() {
			$expect => (),
			found => {
				return Err(ValueError::NumberOfFunctionParamsNotMatching {
					expected: $expect,
					found,
				}
				.into())
			}
		}
	};
}

macro_rules! optional_expect_arguments {
	($arguments: expr, $min: expr, $max: expr) => {
		match $arguments.len() {
			len if ($min..=$max).contains(&len) => (),
			found => {
				return Err(ValueError::NumberOfFunctionParamsNotMatching {
					expected: $min,
					found,
				}
				.into())
			}
		}
	};
}

impl Value {
	pub fn function_if_null(mut arguments: Vec<Self>) -> Result<Self> {
		expect_arguments!(arguments, 2);
		Ok(arguments.remove(0).if_null(arguments.remove(0)))
	}
	pub fn function_null_if(mut arguments: Vec<Self>) -> Result<Self> {
		expect_arguments!(arguments, 2);
		arguments.remove(0).null_if(arguments.remove(0))
	}
	pub fn function_iif(mut arguments: Vec<Self>) -> Result<Self> {
		expect_arguments!(arguments, 3);
		arguments
			.remove(0)
			.iif(arguments.remove(0), arguments.remove(0))
	}
	pub fn function_to_lowercase(mut arguments: Vec<Self>) -> Result<Self> {
		expect_arguments!(arguments, 1);
		arguments.remove(0).to_lowercase()
	}
	pub fn function_to_uppercase(mut arguments: Vec<Self>) -> Result<Self> {
		expect_arguments!(arguments, 1);
		arguments.remove(0).to_uppercase()
	}
	pub fn function_left(mut arguments: Vec<Self>) -> Result<Self> {
		expect_arguments!(arguments, 2);
		arguments.remove(0).left(arguments.remove(0))
	}
	pub fn function_right(mut arguments: Vec<Self>) -> Result<Self> {
		expect_arguments!(arguments, 2);
		arguments.remove(0).right(arguments.remove(0))
	}
	pub fn function_length(mut arguments: Vec<Self>) -> Result<Self> {
		expect_arguments!(arguments, 1);
		arguments.remove(0).length()
	}

	pub fn function_concat(mut arguments: Vec<Self>) -> Result<Self> {
		arguments.remove(0).concat(arguments)
	}

	pub fn function_replace(mut arguments: Vec<Self>) -> Result<Self> {
		expect_arguments!(arguments, 3);
		arguments
			.remove(0)
			.replace(arguments.remove(0), arguments.remove(0))
	}

	pub fn function_round(mut arguments: Vec<Self>) -> Result<Self> {
		optional_expect_arguments!(arguments, 1, 2);
		let value = arguments.remove(0);
		let places = if !arguments.is_empty() {
			arguments.remove(0)
		} else {
			Self::I64(0)
		};
		value.round(places)
	}

	pub fn function_rand(arguments: Vec<Self>) -> Result<Self> {
		match arguments.len() {
			0 => Self::function_random_float(arguments),
			2 => Self::function_random_int(arguments),
			found => {
				Err(ValueError::NumberOfFunctionParamsNotMatching { expected: 0, found }.into())
			}
		}
	}
	pub fn function_random_float(arguments: Vec<Self>) -> Result<Self> {
		expect_arguments!(arguments, 0);
		Ok(Self::F64(fastrand::f64()))
	}
	pub fn function_random_int(mut arguments: Vec<Self>) -> Result<Self> {
		expect_arguments!(arguments, 2);
		let min: i64 = arguments.remove(0).cast()?;
		let max: i64 = arguments.remove(0).cast()?;
		Ok(Self::I64(fastrand::i64(min..=max)))
	}
	pub fn function_uuid(arguments: Vec<Self>) -> Result<Self> {
		expect_arguments!(arguments, 0);
		Ok(Self::Str(Uuid::new_v4().as_hyphenated().to_string())) // TODO: Custom type
	}

	pub fn function_pow(mut arguments: Vec<Self>) -> Result<Self> {
		expect_arguments!(arguments, 2);
		arguments.remove(0).pow(arguments.remove(0))
	}

	pub fn function_convert(mut arguments: Vec<Self>) -> Result<Self> {
		optional_expect_arguments!(arguments, 2, 3);
		let datatype: String = arguments.remove(0).convert()?;
		let value = arguments.remove(0);
		let rule = if !arguments.is_empty() {
			arguments.remove(0)
		} else {
			Self::I64(0)
		};
		Ok(match datatype.to_uppercase().as_str() {
			// Unfortunatly we cannot get datatype directly, it needs to be given as string
			"BOOLEAN" => Value::Bool(value.cast_with_rule(rule)?),
			"INTEGER" => Value::I64(value.cast_with_rule(rule)?),
			"FLOAT" => Value::F64(value.cast_with_rule(rule)?),
			"TEXT" => Value::Str(value.cast_with_rule(rule)?),
			"TIMESTAMP" => {
				// Temp, need Value::Timestamp
				let datetime: NaiveDateTime = value.cast_with_rule(rule)?;

				Value::I64(datetime.timestamp())
			}
			_ => return Err(ValueError::UnimplementedConvert.into()),
		})
	}
	pub fn function_try_convert(arguments: Vec<Self>) -> Result<Self> {
		Ok(Value::function_convert(arguments).unwrap_or(Value::NULL))
	}
}
