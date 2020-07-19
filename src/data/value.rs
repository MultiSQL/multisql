use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt::Debug;
use thiserror::Error as ThisError;

use sqlparser::ast::{DataType, Value as AstValue};

use crate::result::{Error, Result};

#[derive(ThisError, Debug, PartialEq)]
pub enum ValueError {
    #[error("sql type not supported yet")]
    SqlTypeNotSupported,

    #[error("literal not supported yet")]
    LiteralNotSupported,

    #[error("failed to parse number")]
    FailedToParseNumber,

    #[error("add on non numeric value")]
    AddOnNonNumeric,

    #[error("subtract on non numeric value")]
    SubtractOnNonNumeric,

    #[error("multiply on non numeric value")]
    MultiplyOnNonNumeric,

    #[error("divide on non numeric value")]
    DivideOnNonNumeric,

    #[error("null value on not null field")]
    NullValueOnNotNullField,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Bool(bool),
    I64(i64),
    F64(f64),
    Str(String),
    OptBool(Option<bool>),
    OptI64(Option<i64>),
    OptF64(Option<f64>),
    OptStr(Option<String>),
    Empty,
}

impl PartialEq<AstValue> for Value {
    fn eq(&self, other: &AstValue) -> bool {
        match (self, other) {
            (Value::Bool(l), AstValue::Boolean(r))
            | (Value::OptBool(Some(l)), AstValue::Boolean(r)) => l == r,
            (Value::I64(l), AstValue::Number(r))
            | (Value::OptI64(Some(l)), AstValue::Number(r)) => match r.parse::<i64>() {
                Ok(r) => l == &r,
                Err(_) => false,
            },
            (Value::F64(l), AstValue::Number(r))
            | (Value::OptF64(Some(l)), AstValue::Number(r)) => match r.parse::<f64>() {
                Ok(r) => l == &r,
                Err(_) => false,
            },
            (Value::Str(l), AstValue::SingleQuotedString(r))
            | (Value::OptStr(Some(l)), AstValue::SingleQuotedString(r)) => l == r,
            (Value::OptBool(None), AstValue::Null)
            | (Value::OptI64(None), AstValue::Null)
            | (Value::OptF64(None), AstValue::Null)
            | (Value::OptStr(None), AstValue::Null) => true,
            _ => false,
        }
    }
}

impl PartialOrd<Value> for Value {
    fn partial_cmp(&self, other: &Value) -> Option<Ordering> {
        match (self, other) {
            (Value::I64(l), Value::I64(r))
            | (Value::OptI64(Some(l)), Value::I64(r))
            | (Value::I64(l), Value::OptI64(Some(r)))
            | (Value::OptI64(Some(l)), Value::OptI64(Some(r))) => Some(l.cmp(r)),
            (Value::F64(l), Value::F64(r))
            | (Value::OptF64(Some(l)), Value::F64(r))
            | (Value::F64(l), Value::OptF64(Some(r)))
            | (Value::OptF64(Some(l)), Value::OptF64(Some(r))) => l.partial_cmp(r),
            (Value::Str(l), Value::Str(r))
            | (Value::OptStr(Some(l)), Value::Str(r))
            | (Value::Str(l), Value::OptStr(Some(r)))
            | (Value::OptStr(Some(l)), Value::OptStr(Some(r))) => Some(l.cmp(r)),
            _ => None,
        }
    }
}

impl PartialOrd<AstValue> for Value {
    fn partial_cmp(&self, other: &AstValue) -> Option<Ordering> {
        match (self, other) {
            (Value::I64(l), AstValue::Number(r))
            | (Value::OptI64(Some(l)), AstValue::Number(r)) => match r.parse::<i64>() {
                Ok(r) => Some(l.cmp(&r)),
                Err(_) => None,
            },
            (Value::F64(l), AstValue::Number(r))
            | (Value::OptF64(Some(l)), AstValue::Number(r)) => match r.parse::<f64>() {
                Ok(r) => l.partial_cmp(&r),
                Err(_) => None,
            },
            (Value::Str(l), AstValue::SingleQuotedString(r))
            | (Value::OptStr(Some(l)), AstValue::SingleQuotedString(r)) => Some(l.cmp(r)),
            _ => None,
        }
    }
}

impl TryFrom<&AstValue> for Value {
    type Error = Error;

    fn try_from(literal: &AstValue) -> Result<Self> {
        match literal {
            AstValue::Number(v) => v
                .parse::<i64>()
                .map_or_else(|_| v.parse::<f64>().map(Value::F64), |v| Ok(Value::I64(v)))
                .map_err(|_| ValueError::FailedToParseNumber.into()),
            AstValue::Boolean(v) => Ok(Value::Bool(*v)),
            _ => Err(ValueError::SqlTypeNotSupported.into()),
        }
    }
}

impl Value {
    pub fn from_data_type(data_type: DataType, nullable: bool, literal: &AstValue) -> Result<Self> {
        match (data_type, literal) {
            (DataType::Int, AstValue::Number(v)) => v
                .parse()
                .map(|v| {
                    if nullable {
                        Value::OptI64(Some(v))
                    } else {
                        Value::I64(v)
                    }
                })
                .map_err(|_| ValueError::FailedToParseNumber.into()),
            (DataType::Float(_), AstValue::Number(v)) => v
                .parse()
                .map(|v| {
                    if nullable {
                        Value::OptF64(Some(v))
                    } else {
                        Value::F64(v)
                    }
                })
                .map_err(|_| ValueError::FailedToParseNumber.into()),
            (DataType::Boolean, AstValue::Boolean(v)) => {
                if nullable {
                    Ok(Value::OptBool(Some(*v)))
                } else {
                    Ok(Value::Bool(*v))
                }
            }
            (DataType::Int, AstValue::Null) => {
                if nullable {
                    Ok(Value::OptI64(None))
                } else {
                    Err(ValueError::NullValueOnNotNullField.into())
                }
            }
            (DataType::Float(_), AstValue::Null) => {
                if nullable {
                    Ok(Value::OptF64(None))
                } else {
                    Err(ValueError::NullValueOnNotNullField.into())
                }
            }
            (DataType::Boolean, AstValue::Null) => {
                if nullable {
                    Ok(Value::OptBool(None))
                } else {
                    Err(ValueError::NullValueOnNotNullField.into())
                }
            }
            _ => Err(ValueError::SqlTypeNotSupported.into()),
        }
    }

    pub fn clone_by(&self, literal: &AstValue) -> Result<Self> {
        match (self, literal) {
            (Value::I64(_), AstValue::Number(v)) => v
                .parse()
                .map(Value::I64)
                .map_err(|_| ValueError::FailedToParseNumber.into()),
            (Value::OptI64(_), AstValue::Number(v)) => v
                .parse()
                .map(|v| Value::OptI64(Some(v)))
                .map_err(|_| ValueError::FailedToParseNumber.into()),
            (Value::OptI64(_), AstValue::Null) => Ok(Value::OptI64(None)),
            (Value::F64(_), AstValue::Number(v)) => v
                .parse()
                .map(Value::F64)
                .map_err(|_| ValueError::FailedToParseNumber.into()),
            (Value::OptF64(_), AstValue::Number(v)) => v
                .parse()
                .map(|v| Value::OptF64(Some(v)))
                .map_err(|_| ValueError::FailedToParseNumber.into()),
            (Value::OptF64(_), AstValue::Null) => Ok(Value::OptF64(None)),
            (Value::Str(_), AstValue::SingleQuotedString(v)) => Ok(Value::Str(v.clone())),
            (Value::OptStr(_), AstValue::SingleQuotedString(v)) => {
                Ok(Value::OptStr(Some(v.clone())))
            }
            (Value::OptStr(_), AstValue::Null) => Ok(Value::OptStr(None)),
            (Value::Bool(_), AstValue::Boolean(v)) => Ok(Value::Bool(*v)),
            (Value::OptBool(_), AstValue::Boolean(v)) => Ok(Value::OptBool(Some(*v))),
            (Value::OptBool(_), AstValue::Null) => Ok(Value::OptBool(None)),
            _ => Err(ValueError::LiteralNotSupported.into()),
        }
    }

    pub fn add(&self, other: &Value) -> Result<Value> {
        use Value::*;

        match (self, other) {
            (I64(a), I64(b)) => Ok(I64(a + b)),
            (I64(a), OptI64(Some(b))) | (OptI64(Some(a)), I64(b)) => Ok(OptI64(Some(a + b))),
            (OptI64(Some(a)), OptI64(Some(b))) => Ok(OptI64(Some(a + b))),
            (OptI64(None), OptI64(Some(a))) | (OptI64(Some(a)), OptI64(None)) => {
                Ok(OptI64(Some(*a)))
            }
            (F64(a), F64(b)) => Ok(F64(a + b)),
            (F64(a), OptF64(Some(b))) | (OptF64(Some(a)), F64(b)) => Ok(OptF64(Some(a + b))),
            _ => Err(ValueError::AddOnNonNumeric.into()),
        }
    }

    pub fn subtract(&self, other: &Value) -> Result<Value> {
        use Value::*;

        match (self, other) {
            (I64(a), I64(b)) => Ok(I64(a - b)),
            (I64(a), OptI64(Some(b))) | (OptI64(Some(a)), I64(b)) => Ok(OptI64(Some(a - b))),
            (F64(a), F64(b)) => Ok(F64(a - b)),
            (F64(a), OptF64(Some(b))) | (OptF64(Some(a)), F64(b)) => Ok(OptF64(Some(a - b))),
            _ => Err(ValueError::SubtractOnNonNumeric.into()),
        }
    }

    pub fn multiply(&self, other: &Value) -> Result<Value> {
        use Value::*;

        match (self, other) {
            (I64(a), I64(b)) => Ok(I64(a * b)),
            (I64(a), OptI64(Some(b))) | (OptI64(Some(a)), I64(b)) => Ok(OptI64(Some(a * b))),
            (F64(a), F64(b)) => Ok(F64(a * b)),
            (F64(a), OptF64(Some(b))) | (OptF64(Some(a)), F64(b)) => Ok(OptF64(Some(a * b))),
            _ => Err(ValueError::MultiplyOnNonNumeric.into()),
        }
    }

    pub fn divide(&self, other: &Value) -> Result<Value> {
        use Value::*;

        match (self, other) {
            (I64(a), I64(b)) => Ok(I64(a / b)),
            (I64(a), OptI64(Some(b))) | (OptI64(Some(a)), I64(b)) => Ok(OptI64(Some(a / b))),
            (F64(a), F64(b)) => Ok(F64(a / b)),
            (F64(a), OptF64(Some(b))) | (OptF64(Some(a)), F64(b)) => Ok(OptF64(Some(a / b))),
            _ => Err(ValueError::DivideOnNonNumeric.into()),
        }
    }
}
