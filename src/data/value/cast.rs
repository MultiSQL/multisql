use {
    super::{Convert, Value, ValueError},
    crate::result::Result,
    std::convert::TryInto,
};

pub trait Cast<Output> {
    fn cast(self) -> Result<Output>;
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
            Value::Str(value) => value.parse().map_err(|_| ValueError::ImpossibleCast)?,
            Value::Null => return Err(ValueError::ImpossibleCast.into()),
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
            Value::Str(value) => value.parse().map_err(|_| ValueError::ImpossibleCast)?,
            Value::Null => return Err(ValueError::ImpossibleCast.into()),
        }))
    }
}
impl Cast<String> for Value {
    fn cast(self) -> Result<String> {
        self.clone().convert().or(Ok(match self {
            Value::Str(value) => value,
            Value::Bool(value) => (if value { "TRUE" } else { "FALSE" }).to_string(),
            Value::I64(value) => value.to_string(),
            Value::F64(value) => value.to_string(),
            Value::Null => String::from("NULL"),
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
