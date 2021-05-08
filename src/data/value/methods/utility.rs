use {
	crate::{Convert, Result, Value, ValueError},
	chrono::{Datelike, NaiveDate, NaiveDateTime, Timelike},
	std::{
		cmp::min,
		convert::TryInto,
		time::{SystemTime, UNIX_EPOCH},
	},
};

macro_rules! protect_null {
	($protect: expr) => {
		if matches!($protect, Value::Null) {
			return Ok($protect);
		}
	};
}

impl Value {
	pub fn if_null(self, alternative: Self) -> Self {
		if !matches!(self, Value::Null) {
			self
		} else {
			alternative
		}
	}
	pub fn null_if(self, evaluate: Self) -> Result<Self> {
		Ok(if self == evaluate { Value::Null } else { self })
	}
	pub fn iif(self, case_true: Self, case_false: Self) -> Result<Self> {
		Ok(if self.convert()? {
			case_true
		} else {
			case_false
		})
	}

	pub fn to_uppercase(self) -> Result<Self> {
		protect_null!(self);
		let string: String = self.convert()?;
		Ok(string.to_uppercase().into())
	}
	pub fn to_lowercase(self) -> Result<Self> {
		protect_null!(self);
		let string: String = self.convert()?;
		Ok(string.to_lowercase().into())
	}
	pub fn left(self, length: Value) -> Result<Value> {
		protect_null!(self);
		protect_null!(length);
		let length: i64 = length.convert()?;
		if length < 0 {
			return Err(ValueError::BadInput(length.into()).into());
		}
		let length: usize = length as usize;
		let string: String = self.convert()?;

		let truncated = string
			.get(..length)
			.map(|result| result.to_string())
			.unwrap_or(string);
		Ok(Value::Str(truncated))
	}
	pub fn right(self, length: Value) -> Result<Value> {
		protect_null!(self);
		protect_null!(length);
		let length: i64 = length.convert()?;
		if length < 0 {
			return Err(ValueError::BadInput(length.into()).into());
		}
		let length: usize = length as usize;
		let string: String = self.convert()?;

		let truncated = string
			.get(string.len() - min(string.len(), length)..)
			.map(|result| result.to_string())
			.unwrap_or(string);
		Ok(Value::Str(truncated))
	}
	pub fn length(self) -> Result<Value> {
		let string: String = self.convert()?;
		Ok(Value::I64(string.len() as i64))
	}

	pub fn concat(self, strings: Vec<Value>) -> Result<Value> {
		strings
			.into_iter()
			.try_fold(self, |all, this| all.string_concat(this))
	}

	pub fn replace(self, from: Value, to: Value) -> Result<Value> {
		protect_null!(self);
		let string: String = self.convert()?;
		let from: String = from.convert()?;
		let to: String = to.convert()?;

		Ok(string.replace(&from, &to).into())
	}

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
	pub fn date_add(self, amount: Value, datetime: Value) -> Result<Value> {
		let datetime: NaiveDateTime = datetime.convert()?;
		let amount: i64 = amount.convert()?;
		let amount: i32 = amount.try_into().map_err(|_| ValueError::DateError)?;

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
					let day: u32 = days.try_into().map_err(|_| ValueError::DateError).unwrap(); //?;
					Ok(Value::I64(
						datetime
							.with_day(day)
							.ok_or(ValueError::DateError)
							.unwrap() //?
							.timestamp(),
					))
				}
			}
			_ => Err(ValueError::BadInput(self).into()),
		}
	}
	pub fn date_from_parts(self, month: Value, day: Value) -> Result<Value> {
		let (year, month, day): (i64, i64, i64) =
			(self.convert()?, month.convert()?, day.convert()?);
		let datetime = NaiveDate::from_ymd(year as i32, month as u32, day as u32).and_hms(0, 0, 0);

		Ok(Value::I64(datetime.timestamp()))
	}

	pub fn round(self, places: Value) -> Result<Value> {
		if matches!(self, Value::Null) {
			return Ok(self);
		}
		let value: f64 = self.convert()?;
		let places: i64 = places.convert()?;
		let raiser: f64 = 10_u32.pow(places as u32).into();
		Ok(Value::F64((value * raiser).round() / raiser))
	}
	pub fn pow(self, power: Value) -> Result<Value> {
		let value: f64 = self.convert()?;
		let power: f64 = power.convert()?;
		Ok(Value::F64(value.powf(power)))
	}
}
