use {
    super::{
        method::{BinaryOperator, UnaryOperator},
        Aggregate, Function, RecipeError,
    },
    crate::{Error, Result},
    sqlparser::ast::{BinaryOperator as AstBinaryOperator, UnaryOperator as AstUnaryOperator},
    std::convert::TryFrom,
};

impl TryFrom<String> for Function {
    type Error = Error;
    fn try_from(try_from: String) -> Result<Self> {
        match try_from.to_uppercase().as_str() {
            "UPPER" => Ok(Function::Upper),
            "LOWER" => Ok(Function::Lower),
            "LEFT" => Ok(Function::Left),
            "RIGHT" => Ok(Function::Right),

            unimplemented => {
                Err(RecipeError::UnimplementedMethod(String::from(unimplemented)).into())
            }
        }
    }
}

impl TryFrom<String> for Aggregate {
    type Error = Error;
    fn try_from(try_from: String) -> Result<Self> {
        match try_from.to_uppercase().as_str() {
            "MIN" => Ok(Aggregate::Min),
            "MAX" => Ok(Aggregate::Max),
            "SUM" => Ok(Aggregate::Sum),
            "AVG" => Ok(Aggregate::Avg),

            unimplemented => {
                Err(RecipeError::UnimplementedMethod(String::from(unimplemented)).into())
            }
        }
    }
}

impl TryFrom<AstUnaryOperator> for UnaryOperator {
    type Error = Error;
    fn try_from(try_from: AstUnaryOperator) -> Result<Self> {
        match try_from {
            AstUnaryOperator::Plus => Ok(UnaryOperator::Plus),
            AstUnaryOperator::Minus => Ok(UnaryOperator::Minus),
            AstUnaryOperator::Not => Ok(UnaryOperator::Not),

            unimplemented => {
                Err(RecipeError::UnimplementedMethod(format!("{:?}", unimplemented)).into())
            }
        }
    }
}

impl TryFrom<AstBinaryOperator> for BinaryOperator {
    type Error = Error;
    fn try_from(try_from: AstBinaryOperator) -> Result<Self> {
        match try_from {
            AstBinaryOperator::Plus => Ok(BinaryOperator::Plus),
            AstBinaryOperator::Minus => Ok(BinaryOperator::Minus),
            AstBinaryOperator::Multiply => Ok(BinaryOperator::Multiply),
            AstBinaryOperator::Divide => Ok(BinaryOperator::Divide),
            AstBinaryOperator::Modulus => Ok(BinaryOperator::Modulus),

            AstBinaryOperator::And => Ok(BinaryOperator::And),
            AstBinaryOperator::Or => Ok(BinaryOperator::Or),
            AstBinaryOperator::Eq => Ok(BinaryOperator::Eq),
            AstBinaryOperator::Gt => Ok(BinaryOperator::Gt),
            AstBinaryOperator::Lt => Ok(BinaryOperator::Lt),
            AstBinaryOperator::GtEq => Ok(BinaryOperator::GtEq),
            AstBinaryOperator::LtEq => Ok(BinaryOperator::LtEq),

            AstBinaryOperator::StringConcat => Ok(BinaryOperator::StringConcat),

            unimplemented => {
                Err(RecipeError::UnimplementedMethod(format!("{:?}", unimplemented)).into())
            }
        }
    }
}
