use {
    super::{MethodRecipeSolution, RecipeError},
    crate::Value,
    serde::Serialize,
    std::fmt::Debug,
    thiserror::Error,
};

#[derive(Error, Serialize, Debug, PartialEq)]
pub enum CalculationError {
    #[error("wrong type/s used: {0}")]
    WrongType(String),

    #[error(
        "number of function parameters not matching for function: {function:?}; expected: {expected:?}, found: {found:?}"
    )]
    WrongNumberOfArguments {
        function: Function,
        expected: usize,
        found: usize,
    },

    #[error(
        "data types for function: {function:?} wrong, expected: {expected:?}, found: {found:?}"
    )]
    FunctionRequiresDataType {
        function: Function,
        expected: Value,
        found: Value,
    },

    #[error("attempted arithmetic on non numeric types")]
    ArithmeticOnNonNumeric,
    #[error("attempted functionality with incompatible data types")]
    IncompatibleDataTypes,

    #[error("this should be impossible, please report")]
    Unreachable,

    #[error("function: {0:?} failed: {1}")]
    FailedFunction(Function, String),

    #[error("function: {0:?} expects: {1}, got: {2:?}")]
    BadInput(Function, String, Value),

    #[error("other failure occurred: {0}")]
    Failed(String),
}
#[derive(Debug, PartialEq, Serialize, Clone)]
pub enum BooleanCheck {
    IsNull,
}

#[derive(Debug, PartialEq, Serialize, Clone)]
pub enum UnaryOperator {
    Plus,
    Minus,
    Not,
}

#[derive(Debug, PartialEq, Serialize, Clone)]
pub enum BinaryOperator {
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulus,

    And,
    Or,

    Eq,
    NotEq,
    Gt,
    GtEq,
    Lt,
    LtEq,

    StringConcat,
}

#[derive(Debug, PartialEq, Serialize, Clone)]
pub enum Function {
    Upper,
    Lower,

    Left,
    Right,
}

#[derive(Debug, PartialEq, Serialize, Clone)]
pub enum Aggregate {
    Min,
    Max,
    Sum,
    Avg,
}

impl BooleanCheck {
    pub fn solve(self, value: Value) -> MethodRecipeSolution {
        Ok(Value::Bool(match self {
            BooleanCheck::IsNull => matches!(value, Value::Null),
        }))
    }
}

impl UnaryOperator {
    pub fn solve(self, value: Value) -> MethodRecipeSolution {
        match self {
            UnaryOperator::Plus => value.unary_plus(),
            UnaryOperator::Minus => value.unary_minus(),
            UnaryOperator::Not => value.not(),
        }
    }
}

impl BinaryOperator {
    pub fn solve(self, left: Value, right: Value) -> MethodRecipeSolution {
        macro_rules! arithmetic_operation {
            ($left: expr, $right: expr) => {
                match self {
                    BinaryOperator::Plus => $left + $right,
                    BinaryOperator::Minus => $left - $right,
                    BinaryOperator::Multiply => $left * $right,
                    BinaryOperator::Divide => $left / $right,
                    BinaryOperator::Modulus => $left % $right,
                    _ => unreachable!(),
                }
            };
        }
        match self {
            BinaryOperator::Plus
            | BinaryOperator::Minus
            | BinaryOperator::Multiply
            | BinaryOperator::Divide
            | BinaryOperator::Modulus => Ok(match (left, right) {
                (Value::I64(left), Value::I64(right)) => {
                    Value::I64(arithmetic_operation!(left, right))
                }
                (Value::F64(left), Value::F64(right)) => {
                    Value::F64(arithmetic_operation!(left, right))
                }

                #[cfg(feature = "implicit_numeric_conversion")]
                (Value::I64(left), Value::F64(right)) => {
                    Value::F64(arithmetic_operation!(left as f64, right))
                }
                #[cfg(feature = "implicit_numeric_conversion")]
                (Value::F64(left), Value::I64(right)) => {
                    Value::F64(arithmetic_operation!(left, right as f64))
                }
                #[cfg(not(feature = "implicit_numeric_conversion"))]
                (Value::I64(_), Value::F64(_)) | (Value::F64(_), Value::I64(_)) => {
                    return Err(CalculationError::IncompatibleDataTypes);
                }

                (Value::Null, Value::I64(_))
                | (Value::Null, Value::F64(_))
                | (Value::I64(_), Value::Null)
                | (Value::F64(_), Value::Null)
                | (Value::Null, Value::Null) => Value::Null,

                _ => return Err(CalculationError::ArithmeticOnNonNumeric.into()),
            }),

            BinaryOperator::And => {
                if let (Value::Bool(left), Value::Bool(right)) = (left, right) {
                    Ok(Value::Bool(left && right))
                } else {
                    Err(CalculationError::WrongType(String::from(
                        "Binary Boolean Operation on non boolean/s",
                    ))
                    .into())
                }
            }
            BinaryOperator::Or => {
                if let (Value::Bool(left), Value::Bool(right)) = (left, right) {
                    Ok(Value::Bool(left || right))
                } else {
                    Err(CalculationError::WrongType(
                        String::from("Binary Boolean Operation on non boolean/s").into(),
                    )
                    .into())
                }
            }

            BinaryOperator::Eq => Ok(Value::Bool(left == right)),
            BinaryOperator::NotEq => Ok(Value::Bool(left != right)),
            BinaryOperator::Gt => Ok(Value::Bool(left > right)),
            BinaryOperator::GtEq => Ok(Value::Bool(left >= right)),
            BinaryOperator::Lt => Ok(Value::Bool(left < right)),
            BinaryOperator::LtEq => Ok(Value::Bool(left <= right)),

            BinaryOperator::StringConcat => {
                if let (Value::Str(left), Value::Str(right)) = (left, right) {
                    Ok(Value::Str(format!("{}{}", left, right)))
                } else {
                    Err(CalculationError::WrongType(
                        String::from("string concatenation on non string/s").into(),
                    )
                    .into())
                }
            }
        }
    }
}

impl Function {
    pub fn solve(self, arguments: Vec<Value>) -> MethodRecipeSolution {
        macro_rules! expect_arguments {
            ($arguments: expr, $expect: expr) => {
                match $arguments.len() {
                    $expect => (),
                    found => {
                        return Err(CalculationError::WrongNumberOfArguments {
                            // TODO: Move this to recipe creation
                            function: self,
                            expected: $expect,
                            found,
                        }
                        .into());
                    }
                }
            };
        }
        match self {
            Function::Upper | Function::Lower => {
                expect_arguments!(arguments, 1);
                let argument = arguments.get(0).ok_or(CalculationError::Unreachable)?;
                if let Value::Str(argument) = argument {
                    Ok(Value::Str(match self {
                        Function::Upper => argument.to_uppercase(),
                        Function::Lower => argument.to_lowercase(),
                        _ => unreachable!(),
                    }))
                } else if matches!(argument, Value::Null) {
                    Ok(Value::Null)
                } else {
                    Err(CalculationError::FunctionRequiresDataType {
                        function: self,
                        expected: Value::Str(String::new()),
                        found: argument.clone(),
                    }
                    .into())
                }
            }
            Function::Left | Function::Right => {
                expect_arguments!(arguments, 2);
                let (text, length) = (
                    arguments.get(0).ok_or(CalculationError::Unreachable)?,
                    arguments.get(1).ok_or(CalculationError::Unreachable)?,
                );
                if let Value::Str(string) = text {
                    if let Value::I64(length) = length {
                        if length < &0 {
                            return Err(CalculationError::BadInput(
                                self,
                                String::from("positive integer only"),
                                Value::I64(*length),
                            )
                            .into());
                        }
                        let length = *length as usize;
                        Ok(match self {
                            Function::Left => string.get(..length),
                            Function::Right => string.get(
                                (if length > string.len() {
                                    0
                                } else {
                                    string.len() - length
                                })..,
                            ),
                            _ => return Err(CalculationError::Unreachable.into()),
                        }
                        .map(|value| Value::Str(value.into()))
                        .unwrap_or(text.clone()))
                    } else if matches!(length, Value::Null) {
                        Ok(Value::Null)
                    } else {
                        Err(CalculationError::FunctionRequiresDataType {
                            function: self,
                            expected: Value::I64(0),
                            found: length.clone(),
                        }
                        .into())
                    }
                } else if matches!(text, Value::Null) {
                    Ok(Value::Null)
                } else {
                    Err(CalculationError::FunctionRequiresDataType {
                        function: self,
                        expected: Value::Str(String::new()),
                        found: text.clone(),
                    }
                    .into())
                }
            }
        }
    }
}
