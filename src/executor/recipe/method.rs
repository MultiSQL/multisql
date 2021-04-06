use {
    super::{MethodRecipeSolution, RecipeError},
    crate::Value,
    serde::Serialize,
    std::cmp::min,
};

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
        match self {
            BinaryOperator::Plus => left.add(&right),
            BinaryOperator::Minus => left.subtract(&right),
            BinaryOperator::Multiply => left.multiply(&right),
            BinaryOperator::Divide => left.divide(&right),

            BinaryOperator::And => {
                if let (Value::Bool(left), Value::Bool(right)) = (left, right) {
                    Ok(Value::Bool(left && right))
                } else {
                    Err(RecipeError::Failed(String::from(
                        "Binary Boolean Operation on non boolean/s",
                    ))
                    .into())
                }
            }
            BinaryOperator::Or => {
                if let (Value::Bool(left), Value::Bool(right)) = (left, right) {
                    Ok(Value::Bool(left || right))
                } else {
                    Err(RecipeError::Failed(
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

            _ => unimplemented!(), // TODO
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
                        return Err(RecipeError::WrongNumberOfArguments {
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
                let argument = arguments.get(0).unwrap();
                if let Value::Str(argument) = argument {
                    Ok(Value::Str(match self {
                        Function::Upper => argument.to_uppercase(),
                        Function::Lower => argument.to_lowercase(),
                        _ => unreachable!(),
                    }))
                } else {
                    Err(RecipeError::FunctionRequiresDataType {
                        function: self,
                        expected: Value::Str(String::new()),
                        found: argument.clone(),
                    }
                    .into())
                }
            }
            Function::Left | Function::Right => {
                expect_arguments!(arguments, 2);
                let (text, length) = (arguments.get(0).unwrap(), arguments.get(1).unwrap());
                if let Value::Str(text) = text {
                    if let Value::I64(length) = length {
                        let length = *length as usize;
                        match self {
                            Function::Left => text.get(..length),
                            Function::Right => text.get(min(length, text.len())..),
                            _ => unreachable!(),
                        }
                        .ok_or(RecipeError::Failed(String::from("Issue with Left/Right")).into())
                        .map(|value| Value::Str(value.into()))
                    } else {
                        Err(RecipeError::FunctionRequiresDataType {
                            function: self,
                            expected: Value::I64(0),
                            found: length.clone(),
                        }
                        .into())
                    }
                } else {
                    Err(RecipeError::FunctionRequiresDataType {
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
