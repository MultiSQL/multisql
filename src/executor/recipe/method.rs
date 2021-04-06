use {crate::Value, std::cmp::min};

enum BooleanCheck {
    IsNull(Recipe),
}

enum UnaryOperator {
    Plus,
    Minus,
    Not,
}

enum BinaryOperator {
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulus,

    And,
    Or,

    Eq,
    Gt,
    Lt,
    GtEq,
    LtEq,

    StringConcat,
}

enum Function {
    Upper,
    Lower,

    Left,
    Right,
}

enum Aggregate {
    Min,
    Max,
    Sum,
    Avg,
}

impl BooleanCheck {
    fn solve(self, value: Value) -> RecipeSolution {
        Value::Bool(match self {
            BooleanCheck::IsNull => matches!(value, Value::Null),
        })
    }
}

impl UnaryOperator {
    fn solve(self, value: Value) -> RecipeSolution {
        match self {
            UnaryOperator::Plus => value.unary_plus(),
            UnaryOperator::Minus => value.unary_minus(),
            UnaryOperator::Not => value.not(),
        }
    }
}

impl BinaryOperator {
    fn solve(self, left: Value, right: Value) -> RecipeSolution {
        match self {
            BinaryOperator::Plus => left.add(right),
            _ => unimplemented!(), // TODO
        }
    }
}

impl Function {
    fn solve(self, arguments: Vec<Value>) -> RecipeSolution {
        macro_rules! expect_arguments {
            ($arguments: expr, $expect: expr) => {
                match $arguments.len() {
                    $expect => (),
                    found => {
                        return Err(EvaluateError::NumberOfFunctionParamsNotMatching {
                            expected: $expect,
                            found,
                        }
                        .into())
                    }
                }
            };
        }
        match function {
            Function::Upper | Function::Lower => {
                expect_arguments!(arguments, 1);
                let argument = arguments[0];
                Some(if let Value::Str(argument) = argument {
                    Ok(match function {
                        Function::Upper => argument.to_uppercase(),
                        Function::Lower => argument.to_lowercase(),
                    })
                } else {
                    Err(EvaluateError::FunctionRequiresStringValue)
                })
            }
            Function::Left | Function::Right => {
                expect_arguments!(arguments, 2);
                let (text, length) = (arguments[0], arguments[1]);
                Some(if let Value::Str(text) = text {
                    if let Value::I64(length) = length {
                        Ok(match function {
                            Function::Left => text.get(..length),
                            Function::Right => text.get(min(length, text.len())..),
                        })
                    } else {
                        Err(EvaluateError::FunctionRequiresIntegerValue(
                            function, length,
                        ))
                    }
                } else {
                    Err(EvaluateError::FunctionRequiresStringValue(function, text))
                })
            }
        }
    }
}
