use {
	crate::{Convert, Result, Value, ValueError},
	chrono::{Datelike, NaiveDate, NaiveDateTime, Timelike},
	fstrings::*,
	std::{
		cmp::min,
		convert::TryInto,
		panic,
		time::{SystemTime, UNIX_EPOCH},
	},
};

macro_rules! protect_null {
	($protect: expr) => {
		match $protect {
			Value::Null => return Ok(Value::Null),
			other => other,
		}
	};
}

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
			len if len >= $min && len <= $max => (),
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
	pub fn function_now(arguments: Vec<Self>) -> Result<Self> {
		expect_arguments!(arguments, 0);
		Value::now()
	}
	pub fn function_year(mut arguments: Vec<Self>) -> Result<Self> {
		expect_arguments!(arguments, 1);
		protect_null!(arguments.remove(0)).year()
	}
	pub fn function_month(mut arguments: Vec<Self>) -> Result<Self> {
		expect_arguments!(arguments, 1);
		protect_null!(arguments.remove(0)).month()
	}
	pub fn function_day(mut arguments: Vec<Self>) -> Result<Self> {
		expect_arguments!(arguments, 1);
		protect_null!(arguments.remove(0)).day()
	}
	pub fn function_hour(mut arguments: Vec<Self>) -> Result<Self> {
		expect_arguments!(arguments, 1);
		protect_null!(arguments.remove(0)).hour()
	}
	pub fn function_minute(mut arguments: Vec<Self>) -> Result<Self> {
		expect_arguments!(arguments, 1);
		protect_null!(arguments.remove(0)).minute()
	}
	pub fn function_second(mut arguments: Vec<Self>) -> Result<Self> {
		expect_arguments!(arguments, 1);
		protect_null!(arguments.remove(0)).second()
	}

	pub fn function_timestamp_add(mut arguments: Vec<Self>) -> Result<Self> {
		expect_arguments!(arguments, 3);
		arguments.remove(0).date_add(
			protect_null!(arguments.remove(0)),
			protect_null!(arguments.remove(0)),
		)
	}
	pub fn function_timestamp_from_parts(arguments: Vec<Self>) -> Result<Self> {
		optional_expect_arguments!(arguments, 1, 6);
		protect_null!(arguments
			.get(0)
			.map(|value| value.clone())
			.unwrap_or(Value::I64(1)))
		.date_from_parts(
			protect_null!(arguments
				.get(1)
				.map(|value| value.clone())
				.unwrap_or(Value::I64(1))),
			protect_null!(arguments
				.get(2)
				.map(|value| value.clone())
				.unwrap_or(Value::I64(1))),
			protect_null!(arguments
				.get(3)
				.map(|value| value.clone())
				.unwrap_or(Value::I64(0))),
			protect_null!(arguments
				.get(4)
				.map(|value| value.clone())
				.unwrap_or(Value::I64(0))),
			protect_null!(arguments
				.get(5)
				.map(|value| value.clone())
				.unwrap_or(Value::I64(0))),
		)
	}
}

// System
impl Value {
	pub fn now() -> Result<Value> {
		Ok(Value::I64(
			NaiveDateTime::from_timestamp(
				SystemTime::now()
					.duration_since(UNIX_EPOCH)
					.unwrap()
					.as_secs() as i64,
				0,
			)
			.timestamp(),
		))
	}
}

// Parts
impl Value {
	pub fn year(self) -> Result<Value> {
		let datetime: NaiveDateTime = self.convert()?;
		Ok(Value::I64(datetime.year() as i64))
	}
	pub fn month(self) -> Result<Value> {
		let datetime: NaiveDateTime = self.convert()?;
		Ok(Value::I64(datetime.month() as i64))
	}
	pub fn day(self) -> Result<Value> {
		let datetime: NaiveDateTime = self.convert()?;
		Ok(Value::I64(datetime.day() as i64))
	}
	pub fn hour(self) -> Result<Value> {
		let datetime: NaiveDateTime = self.convert()?;
		Ok(Value::I64(datetime.hour() as i64))
	}
	pub fn minute(self) -> Result<Value> {
		let datetime: NaiveDateTime = self.convert()?;
		Ok(Value::I64(datetime.minute() as i64))
	}
	pub fn second(self) -> Result<Value> {
		let datetime: NaiveDateTime = self.convert()?;
		Ok(Value::I64(datetime.second() as i64))
	}
}

// Math
impl Value {
	pub fn date_add(self, amount: Value, datetime: Value) -> Result<Value> {
		let datetime: NaiveDateTime = datetime.convert()?;
		let amount: i64 = amount.convert()?;
		let amount: i32 = amount.try_into().map_err(|_| ValueError::DateError)?;
		if amount > 100_000 {
			panic!("Looks like you put the amount and timestamp the wrong way around. This will be fixed in future by using different datatypes");
		}

		match self {
			Value::Str(string) if string == "YEAR" => {
				let years = datetime.year() + amount as i32;
				let calculated = datetime.with_year(years).unwrap_or(
					datetime
						.with_day(28)
						.ok_or(ValueError::DateError)
						.unwrap() //?
						.with_year(years)
						.ok_or(ValueError::DateError)
						.unwrap(), //?,
				);
				Ok(Value::I64(calculated.timestamp()))
			}
			Value::Str(string) if string == "MONTH" => {
				let month: i32 = datetime
					.month()
					.try_into()
					.map_err(|_| ValueError::DateError)?;

				let months = month + amount;
				let month = ((months - 1) % 12) + 1;

				let years = (months - month) / 12;
				let month: u32 = month.try_into().map_err(|_| ValueError::DateError).unwrap(); //?;

				let (years, month) = if month == 0 { (-1, 12) } else { (years, month) }; // TEMP-- no support for > -1 yet

				let next_month = if datetime.month() == 12 {
					NaiveDate::from_ymd(datetime.year() + 1, 1, 1)
				} else {
					NaiveDate::from_ymd(datetime.year(), datetime.month() + 1, 1)
				};
				let this_month = NaiveDate::from_ymd(datetime.year(), datetime.month(), 1);

				let month_days: u32 = NaiveDate::signed_duration_since(next_month, this_month)
					.num_days()
					.try_into()
					.map_err(|_| ValueError::DateError)?;

				let day = min(datetime.day(), month_days);
				let calculated = datetime
					.with_day(day)
					.ok_or(ValueError::DateError)
					.unwrap() //?
					.with_month(month)
					.ok_or(ValueError::DateError)
					.unwrap(); //?;

				let calculated = Value::I64(calculated.timestamp());
				Value::Str(String::from("YEAR")).date_add(Value::I64(years as i64), calculated)
			}
			Value::Str(string) if string == "DAY" => {
				let day: i32 = datetime
					.day()
					.try_into()
					.map_err(|_| ValueError::DateError)?;
				let days = day + amount;

				let next_month = if datetime.month() == 12 {
					NaiveDate::from_ymd(datetime.year() + 1, 1, 1)
				} else {
					NaiveDate::from_ymd(datetime.year(), datetime.month() + 1, 1)
				};
				let this_month = NaiveDate::from_ymd(datetime.year(), datetime.month(), 1);

				let month_days: i32 = NaiveDate::signed_duration_since(next_month, this_month)
					.num_days()
					.try_into()
					.map_err(|_| ValueError::DateError)?;

				if days > month_days {
					let first_day = datetime.with_day(1).ok_or(ValueError::DateError)?;
					let next_month = Value::Str(String::from("MONTH"))
						.date_add(Value::I64(1), Value::I64(first_day.timestamp()))?;
					Value::Str(String::from("DAY")).date_add(
						Value::I64(
							(days - month_days - 1)
								.try_into()
								.map_err(|_| ValueError::DateError)
								.unwrap(), //?,
						),
						next_month,
					)
				} else if days <= 0 {
					let prev_month = if datetime.month() == 1 {
						NaiveDate::from_ymd(datetime.year() - 1, 12, 1)
					} else {
						NaiveDate::from_ymd(datetime.year(), datetime.month() - 1, 1)
					};

					let prev_month_days: i32 =
						NaiveDate::signed_duration_since(this_month, prev_month)
							.num_days()
							.try_into()
							.map_err(|_| ValueError::DateError)?;

					let first_day = datetime.with_day(1).ok_or(ValueError::DateError)?;
					let prev_month = Value::Str(String::from("MONTH"))
						.date_add(Value::I64(-1), Value::I64(first_day.timestamp()))?;
					Value::Str(String::from("DAY")).date_add(
						Value::I64(
							(days + prev_month_days - 1)
								.try_into()
								.map_err(|_| ValueError::DateError)
								.unwrap(), //?,
						),
						prev_month,
					)
				} else {
					let day: u32 = days.try_into().map_err(|_| ValueError::DateError)?;
					Ok(Value::I64(
						datetime
							.with_day(day)
							.ok_or(ValueError::DateError)?
							.timestamp(),
					))
				}
			}
			_ => Err(ValueError::BadInput(self).into()),
		}
	}
	pub fn date_from_parts(
		self,
		month: Value,
		day: Value,
		hour: Value,
		minute: Value,
		second: Value,
	) -> Result<Value> {
		let (year, month, day, hour, minute, second): (i64, i64, i64, i64, i64, i64) = (
			self.convert()?,
			month.convert()?,
			day.convert()?,
			hour.convert()?,
			minute.convert()?,
			second.convert()?,
		);
		let (year, month, day, hour, minute, second): (i32, u32, u32, u32, u32, u32) = (
			year.try_into().map_err(|_| ValueError::DateError)?,
			month.try_into().map_err(|_| ValueError::DateError)?,
			day.try_into().map_err(|_| ValueError::DateError)?,
			hour.try_into().map_err(|_| ValueError::DateError)?,
			minute.try_into().map_err(|_| ValueError::DateError)?,
			second.try_into().map_err(|_| ValueError::DateError)?,
		);
		let datetime = panic::catch_unwind(|| {
			NaiveDate::from_ymd(year, month, day).and_hms(hour, minute, second)
		})
		.map_err(|panic| {
			ValueError::SpecifiedTimestampError(f!(
				"{year=}, {month=}, {day=},\n{hour=}, {minute=}, {second=}\n{panic=:?}"
			))
		})?;

		Ok(Value::I64(datetime.timestamp()))
	}
}
