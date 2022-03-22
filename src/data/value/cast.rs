use {
	super::{Convert, Value, ValueError},
	crate::{Error, Result},
	chrono::{NaiveDate, NaiveDateTime, NaiveTime, ParseError},
	std::convert::TryInto,
	thousands::Separable,
};

pub trait Cast<Output> {
	fn cast(self) -> Result<Output>;
}
pub trait CastWithRules<Output> {
	fn cast_with_rule(self, rule: Self) -> Result<Output>;
}

// Cores
impl Cast<bool> for Value {
	fn cast(self) -> Result<bool> {
		self.clone().convert().or(Ok(match self {
			Value::Bool(value) => value,
			Value::I64(value) => match value {
				1 => true,
				0 => false,
				_ => return Err(ValueError::ImpossibleCast.into()),
			},
			Value::F64(value) => {
				if value.eq(&1.0) {
					true
				} else if value.eq(&0.0) {
					false
				} else {
					return Err(ValueError::ImpossibleCast.into());
				}
			}
			Value::Str(value) => match value.to_uppercase().as_str() {
				"TRUE" => true,
				"FALSE" => false,
				_ => return Err(ValueError::ImpossibleCast.into()),
			},
			Value::Null => return Err(ValueError::ImpossibleCast.into()),
			_ => unimplemented!(),
		}))
	}
}

impl Cast<i64> for Value {
	fn cast(self) -> Result<i64> {
		self.clone().convert().or(Ok(match self {
			Value::Bool(value) => {
				if value {
					1
				} else {
					0
				}
			}
			Value::I64(value) => value,
			Value::F64(value) => value.trunc() as i64,
			Value::Str(value) => lexical::parse(value).map_err(|_| ValueError::ImpossibleCast)?,
			Value::Null => return Err(ValueError::ImpossibleCast.into()),
			_ => unimplemented!(),
		}))
	}
}

impl Cast<f64> for Value {
	fn cast(self) -> Result<f64> {
		self.clone().convert().or(Ok(match self {
			Value::Bool(value) => {
				if value {
					1.0
				} else {
					0.0
				}
			}
			Value::I64(value) => (value as f64).trunc(),
			Value::F64(value) => value,
			Value::Str(value) => fast_float::parse(value).map_err(|_| ValueError::ImpossibleCast)?,
			Value::Null => return Err(ValueError::ImpossibleCast.into()),
			_ => unimplemented!(),
		}))
	}
}
impl Cast<String> for Value {
	fn cast(self) -> Result<String> {
		self.clone().convert().or(Ok(match self {
			Value::Bool(value) => (if value { "TRUE" } else { "FALSE" }).to_string(),
			Value::I64(value) => lexical::to_string(value),
			Value::F64(value) => lexical::to_string(value),
			Value::Str(value) => value,
			Value::Null => String::from("NULL"),
			_ => unimplemented!(),
		}))
	}
}

// Utilities
impl Cast<usize> for Value {
	fn cast(self) -> Result<usize> {
		let int: i64 = self.cast()?;
		int.try_into()
			.map_err(|_| ValueError::ImpossibleCast.into())
	}
}

// Non-Core
impl CastWithRules<bool> for Value {
	fn cast_with_rule(self, rule: Self) -> Result<bool> {
		match rule {
			Value::I64(000) | Value::Bool(true) => self.cast(),
			_ => Err(ValueError::InvalidConversionRule.into()),
		}
	}
}
impl CastWithRules<i64> for Value {
	fn cast_with_rule(self, rule: Self) -> Result<i64> {
		match rule {
			Value::I64(000) | Value::Bool(true) => self.cast(),
			_ => Err(ValueError::InvalidConversionRule.into()),
		}
	}
}
impl CastWithRules<f64> for Value {
	fn cast_with_rule(self, rule: Self) -> Result<f64> {
		match rule {
			Value::I64(000) | Value::Bool(true) => self.cast(),
			_ => Err(ValueError::InvalidConversionRule.into()),
		}
	}
}
impl CastWithRules<String> for Value {
	fn cast_with_rule(self, rule: Self) -> Result<String> {
		match rule {
			Value::I64(000) | Value::Bool(true) => self.cast(),
			Value::Str(specified) if specified == String::from("DATETIME") => {
				Ok(NaiveDateTime::from_timestamp(self.convert()?, 0)
					.format("%F %T")
					.to_string())
			}
			Value::Str(specified) if specified == String::from("MONEY") => {
				let value: f64 = self.convert()?;
				let value = (value * 100.0).round() / 100.0;
				let value = value.separate_with_commas();
				Ok(format!("${}", value))
			}
			Value::Str(specified) if specified == String::from("SEPARATED") => {
				let value: f64 = self.convert()?;
				let value = (value * 100.0).round() / 100.0;
				let value = value.separate_with_commas();
				Ok(format!("{}", value))
			}
			Value::Str(format) if matches!(self, Value::I64(..)) => {
				// TODO: TIMESTAMP type
				Ok(NaiveDateTime::from_timestamp(self.convert()?, 0)
					.format(&format)
					.to_string())
			}
			_ => Err(ValueError::InvalidConversionRule.into()),
		}
	}
}

// Non-SQL
// - DateTime
fn parse_error_into(error: ParseError) -> Error {
	ValueError::DateTimeParseError(format!("{:?}", error)).into()
}
impl Cast<NaiveDateTime> for Value {
	// Default (from Timestamp)
	fn cast(self) -> Result<NaiveDateTime> {
		let timestamp: i64 = self.cast()?;
		NaiveDateTime::from_timestamp_opt(timestamp, 0).ok_or(ValueError::ImpossibleCast.into())
	}
}
impl CastWithRules<NaiveDateTime> for Value {
	fn cast_with_rule(self, rule: Self) -> Result<NaiveDateTime> {
		fn for_format_datetime(string: Value, format: &str) -> Result<NaiveDateTime> {
			let string: String = string.cast()?;
			let string: &str = string.as_str();
			NaiveDateTime::parse_from_str(string, format).map_err(parse_error_into)
		}
		fn for_format_date(string: Value, format: &str) -> Result<NaiveDateTime> {
			let string: String = string.cast()?;
			let string: &str = string.as_str();
			Ok(NaiveDate::parse_from_str(string, format)
				.map_err(parse_error_into)?
				.and_hms(0, 0, 0))
		}
		fn for_format_time(string: Value, format: &str) -> Result<NaiveDateTime> {
			let string: String = string.cast()?;
			let string: &str = string.as_str();
			Ok(NaiveDateTime::from_timestamp(0, 0)
				.date()
				.and_time(NaiveTime::parse_from_str(string, format).map_err(parse_error_into)?))
		}
		fn try_rules(try_value: &Value, rules: &[i64]) -> Result<NaiveDateTime> {
			rules
				.iter()
				.find_map(|try_rule| try_value.clone().cast_with_rule((*try_rule).into()).ok())
				.ok_or(ValueError::ParseError(try_value.clone(), "TIMESTAMP").into())
		}
		const TRY_RULES_TIMESTAMP: [i64; 1] = [000];
		const TRY_RULES_DATETIME: [i64; 7] = [010, 011, 020, 021, 030, 031, 060];
		const TRY_RULES_DATE: [i64; 4] = [022, 033, 032, 061]; // 033 should go before 032
		const TRY_RULES_TIME: [i64; 2] = [100, 101];

		match rule {
			Value::Bool(true) => try_rules(&self, &TRY_RULES_TIMESTAMP),
			Value::Str(custom) => match custom.as_str() {
				"TIMESTAMP" => try_rules(&self, &TRY_RULES_TIMESTAMP),
				"DATETIME" => try_rules(&self, &TRY_RULES_DATETIME),
				"DATE" => try_rules(&self, &TRY_RULES_DATE),
				"TIME" => try_rules(&self, &TRY_RULES_TIME),
				custom_format => for_format_datetime(self.clone(), custom_format)
					.or(for_format_date(self.clone(), custom_format))
					.or(for_format_time(self, custom_format)),
			},
			Value::I64(000) => {
				// From Timestamp (Default)
				self.cast()
			}
			// 01* - Statically specifically defined by accepted standards bodies
			/*Value::I64(010) => {
				// From RFC 3339 format
				let datetime_string: String = self.cast()?;
				DateTime::parse_from_rfc3339(datetime_string.as_str()).map_err(parse_error_into)
			}
			Value::I64(011) => {
				// From RFC 2822 format
				let datetime_string: String = self.cast()?;
				DateTime::parse_from_rfc2822(datetime_string.as_str()).map_err(parse_error_into)
			}*/
			// 02* - Conventional
			// - From Database format (YYYY-MM-DD HH:MM:SS)
			Value::I64(020) => for_format_datetime(self, "%F %T"),
			// - From Database format, no seconds (YYYY-MM-DD HH:MM)
			Value::I64(021) => for_format_datetime(self, "%F %R"),
			// - From Database format, no time (YYYY-MM-DD)
			Value::I64(022) => for_format_date(self, "%F"),

			// 0(3-4)* - Normal
			// - From Database format, grossified time (YYYY-MM-DD HH:MM:SS (AM/PM))
			Value::I64(030) => for_format_datetime(self, "%F %r"),
			// - From Database format, grossified time, no seconds (YYYY-MM-DD HH:MM (AM/PM))
			Value::I64(031) => for_format_datetime(self, "%I:%M %p"),
			// - From dd-Mon-YYYY
			Value::I64(032) => for_format_date(self, "%v"),
			// - From dd-Mon-YY
			Value::I64(033) => for_format_date(self, "%e-%b-%y"),

			// 0(5-8)* - Locales
			// 06* - Australia
			Value::I64(060) => for_format_datetime(self, "%d/%m/%Y %H:%M"),
			Value::I64(061) => for_format_date(self, "%d/%m/%Y"),
			// (TODO(?))

			// 10* - Time
			// - (HH:MM:SS)
			Value::I64(100) => for_format_time(self, "%T"),
			// - No seconds (HH:MM)
			Value::I64(101) => for_format_time(self, "%R"),
			_ => Err(ValueError::InvalidConversionRule.into()),
		}
	}
}
