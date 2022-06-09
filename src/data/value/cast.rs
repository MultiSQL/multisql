use {
	super::{Value, ValueError},
	crate::{Error, Result},
	//chrono::{NaiveDate, NaiveDateTime, NaiveTime, ParseError},
	enum_dispatch::enum_dispatch,
	std::convert::TryInto,
	//thousands::Separable,
};

#[enum_dispatch(Value)]
pub trait Cast<Output: Sized>
where
	Self: Sized,
{
	fn cast(self) -> Result<Output> {
		Err(Error::Value(ValueError::UnimplementedCast))
	}
}
pub trait CastWithRules<Output> {
	fn cast_with_rule(self, rule: Self) -> Result<Output>;
}

impl<T> Cast<T> for T {
	fn cast(self) -> Result<T> {
		Ok(self)
	}
}

/*impl<T> Cast<T> for Null {
	fn cast(self) -> Result<T> {
		Err(ValueError::ImpossibleCast.into())
	}
}*/

impl Cast<bool> for i64 {
	fn cast(self) -> Result<bool> {
		match self {
			1 => Ok(true),
			0 => Ok(false),
			_ => Err(ValueError::ImpossibleCast.into()),
		}
	}
}
impl Cast<bool> for u64 {
	fn cast(self) -> Result<bool> {
		match self {
			1 => Ok(true),
			0 => Ok(false),
			_ => Err(ValueError::ImpossibleCast.into()),
		}
	}
}
impl Cast<bool> for f64 {
	fn cast(self) -> Result<bool> {
		if self.eq(&1.0) {
			Ok(true)
		} else if self.eq(&0.0) {
			Ok(false)
		} else {
			Err(ValueError::ImpossibleCast.into())
		}
	}
}
impl Cast<bool> for String {
	fn cast(self) -> Result<bool> {
		match self.to_uppercase().as_str() {
			"TRUE" => Ok(true),
			"FALSE" => Ok(false),
			_ => Err(ValueError::ImpossibleCast.into()),
		}
	}
}

// Cores
impl Cast<u64> for bool {
	fn cast(self) -> Result<u64> {
		Ok(if self { 1 } else { 0 })
	}
}
impl Cast<u64> for i64 {
	fn cast(self) -> Result<u64> {
		self.try_into()
			.map_err(|_| Error::Value(ValueError::ImpossibleCast))
	}
}
impl Cast<u64> for f64 {
	fn cast(self) -> Result<u64> {
		let int: i64 = self.cast()?;
		int.cast()
	}
}
impl Cast<u64> for String {
	fn cast(self) -> Result<u64> {
		lexical::parse(self).map_err(|_| Error::Value(ValueError::ImpossibleCast))
	}
}

impl Cast<i64> for bool {
	fn cast(self) -> Result<i64> {
		Ok(if self { 1 } else { 0 })
	}
}
impl Cast<i64> for u64 {
	fn cast(self) -> Result<i64> {
		self.try_into()
			.map_err(|_| Error::Value(ValueError::ImpossibleCast))
	}
}
impl Cast<i64> for f64 {
	fn cast(self) -> Result<i64> {
		Ok(self.trunc() as i64) // TODO: Better
	}
}
impl Cast<i64> for String {
	fn cast(self) -> Result<i64> {
		lexical::parse(self).map_err(|_| Error::Value(ValueError::ImpossibleCast))
	}
}

impl Cast<f64> for bool {
	fn cast(self) -> Result<f64> {
		Ok(if self { 1.0 } else { 0.0 })
	}
}
impl Cast<f64> for u64 {
	fn cast(self) -> Result<f64> {
		Ok((self as f64).trunc())
	}
}
impl Cast<f64> for i64 {
	fn cast(self) -> Result<f64> {
		Ok((self as f64).trunc())
	}
}
impl Cast<f64> for String {
	fn cast(self) -> Result<f64> {
		fast_float::parse(self).map_err(|_| Error::Value(ValueError::ImpossibleCast))
	}
}

impl Cast<String> for bool {
	fn cast(self) -> Result<String> {
		Ok((if self { "TRUE" } else { "FALSE" }).to_string())
	}
}
impl Cast<String> for u64 {
	fn cast(self) -> Result<String> {
		Ok(lexical::to_string(self))
	}
}
impl Cast<String> for i64 {
	fn cast(self) -> Result<String> {
		Ok(lexical::to_string(self))
	}
}
impl Cast<String> for f64 {
	fn cast(self) -> Result<String> {
		Ok(lexical::to_string(self))
	}
}

// Utilities
impl<T: Cast<u64>> Cast<usize> for T {
	fn cast(self) -> Result<usize> {
		let int: u64 = self.cast()?;
		int.try_into()
			.map_err(|_| ValueError::ImpossibleCast.into())
	}
}

/*
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
*/
/*impl CastWithRules<String> for Value {
	fn cast_with_rule(self, rule: Self) -> Result<String> {
		match rule {
			Value::I64(000) | Value::Bool(true) => self.cast(),
			Value::Str(specified) if specified == *"DATETIME" => {
				Ok(NaiveDateTime::from_timestamp(self.convert()?, 0)
					.format("%F %T")
					.to_string())
			}
			Value::Str(specified) if specified == *"MONEY" => {
				let value: f64 = self.convert()?;
				let value = (value * 100.0).round() / 100.0;
				let value = value.separate_with_commas();
				Ok(format!("${}", value))
			}
			Value::Str(specified) if specified == *"SEPARATED" => {
				let value: f64 = self.convert()?;
				let value = (value * 100.0).round() / 100.0;
				let value = value.separate_with_commas();
				Ok(value)
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
}*/

/*
// Non-SQL
// - DateTime
fn parse_error_into(error: ParseError) -> Error {
	ValueError::DateTimeParseError(format!("{:?}", error)).into()
}
impl Cast<NaiveDateTime> for Value {
	// Default (from Timestamp)
	fn cast(self) -> Result<NaiveDateTime> {
		let timestamp: i64 = self.cast()?;
		NaiveDateTime::from_timestamp_opt(timestamp, 0)
			.ok_or_else(|| ValueError::ImpossibleCast.into())
	}
}
#[allow(clippy::zero_prefixed_literal)]
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
				.ok_or_else(|| ValueError::ParseError(try_value.clone(), "TIMESTAMP").into())
		}
		const TRY_RULES_TIMESTAMP: [i64; 1] = [000];
		const TRY_RULES_DATETIME: [i64; 9] = [010, 011, 020, 021, 030, 031, 060, 062, 063];
		const TRY_RULES_DATE: [i64; 6] = [022, 033, 032, 061, 064, 040]; // 033 should go before 032
		const TRY_RULES_TIME: [i64; 2] = [100, 101];

		match rule {
			Value::Null => try_rules(&self, &TRY_RULES_TIMESTAMP)
				.or_else(|_| try_rules(&self, &TRY_RULES_DATETIME))
				.or_else(|_| try_rules(&self, &TRY_RULES_DATE))
				.or_else(|_| try_rules(&self, &TRY_RULES_TIME)),
			Value::Bool(true) => try_rules(&self, &TRY_RULES_TIMESTAMP),
			Value::Str(custom) => match custom.as_str() {
				"TIMESTAMP" => try_rules(&self, &TRY_RULES_TIMESTAMP),
				"DATETIME" => try_rules(&self, &TRY_RULES_DATETIME),
				"DATE" => try_rules(&self, &TRY_RULES_DATE),
				"TIME" => try_rules(&self, &TRY_RULES_TIME),
				custom_format => for_format_datetime(self.clone(), custom_format)
					.or_else(|_| for_format_date(self.clone(), custom_format))
					.or_else(|_| for_format_time(self, custom_format)),
			},
			Value::I64(000) => {
				// From Timestamp (Default)
				self.cast()
			}
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

			Value::I64(040) => for_format_date(self, "%Y%m%d"),

			// 0(5-8)* - Locales
			// 06* - Australia
			Value::I64(060) => for_format_datetime(self, "%d/%m/%Y %H:%M"),
			Value::I64(061) => for_format_date(self, "%d/%m/%Y"),
			Value::I64(062) => for_format_datetime(self, "%d/%m/%Y %H:%M:%S"),
			Value::I64(063) => for_format_datetime(self, "%d%m%Y %H:%M:%S"),
			Value::I64(064) => for_format_date(self, "%d%m%Y"),

			// 10* - Time
			// - (HH:MM:SS)
			Value::I64(100) => for_format_time(self, "%T"),
			// - No seconds (HH:MM)
			Value::I64(101) => for_format_time(self, "%R"),
			_ => Err(ValueError::InvalidConversionRule.into()),
		}
	}
}
*/
