use {
    super::{AggregateOperator, BinaryOperator, FunctionOperator, RecipeError, UnaryOperator},
    crate::{Result, Value},
    sqlparser::ast::{BinaryOperator as AstBinaryOperator, UnaryOperator as AstUnaryOperator},
};

pub trait TryIntoMethod<MethodType> {
    fn into_method(self) -> Result<MethodType>;
}

impl TryIntoMethod<FunctionOperator> for String {
    fn into_method(self) -> Result<FunctionOperator> {
        match self.to_uppercase().as_str() {
            "UPPER" => Ok(Value::function_to_uppercase),
            "LOWER" => Ok(Value::function_to_lowercase),

            "LEFT" => Ok(Value::function_left),
            "RIGHT" => Ok(Value::function_right),

            "IIF" => Ok(Value::function_iif),
            "IFNULL" => Ok(Value::function_if_null),

            unimplemented => {
                Err(RecipeError::UnimplementedMethod(String::from(unimplemented)).into())
            }
        }
    }
}

impl TryIntoMethod<AggregateOperator> for String {
    fn into_method(self) -> Result<AggregateOperator> {
        match self.to_uppercase().as_str() {
            "COUNT" => Ok(Value::aggregate_count),
            "MIN" => Ok(Value::aggregate_min),
            "MAX" => Ok(Value::aggregate_max),
            "SUM" => Ok(Value::aggregate_sum),

            unimplemented => {
                Err(RecipeError::UnimplementedMethod(String::from(unimplemented)).into())
            }
        }
    }
}

impl TryIntoMethod<UnaryOperator> for AstUnaryOperator {
    fn into_method(self) -> Result<UnaryOperator> {
        match self {
            AstUnaryOperator::Plus => Ok(Value::generic_unary_plus),
            AstUnaryOperator::Minus => Ok(Value::generic_unary_minus),
            AstUnaryOperator::Not => Ok(Value::not),

            unimplemented => {
                Err(RecipeError::UnimplementedMethod(format!("{:?}", unimplemented)).into())
            }
        }
    }
}

impl TryIntoMethod<BinaryOperator> for AstBinaryOperator {
    fn into_method(self) -> Result<BinaryOperator> {
        match self {
            AstBinaryOperator::Plus => Ok(Value::generic_add),
            AstBinaryOperator::Minus => Ok(Value::generic_subtract),
            AstBinaryOperator::Multiply => Ok(Value::generic_multiply),
            AstBinaryOperator::Divide => Ok(Value::generic_divide),
            AstBinaryOperator::Modulus => Ok(Value::generic_modulus),

            AstBinaryOperator::And => Ok(Value::and),
            AstBinaryOperator::Or => Ok(Value::or),

            AstBinaryOperator::Eq => Ok(Value::eq),
            AstBinaryOperator::NotEq => Ok(Value::not_eq),
            AstBinaryOperator::Gt => Ok(Value::gt),
            AstBinaryOperator::GtEq => Ok(Value::gt_eq),
            AstBinaryOperator::Lt => Ok(Value::lt),
            AstBinaryOperator::LtEq => Ok(Value::lt_eq),

            AstBinaryOperator::StringConcat => Ok(Value::string_concat),

            unimplemented => {
                Err(RecipeError::UnimplementedMethod(format!("{:?}", unimplemented)).into())
            }
        }
    }
}
