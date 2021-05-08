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
            "CONVERT" => Ok(Value::function_convert),
            "TRY_CONVERT" => Ok(Value::function_try_convert),

            "UPPER" => Ok(Value::function_to_uppercase),
            "LOWER" => Ok(Value::function_to_lowercase),

            "LEFT" => Ok(Value::function_left),
            "RIGHT" => Ok(Value::function_right),

            "LEN" => Ok(Value::function_length),
            "CONCAT" => Ok(Value::function_concat),
            "REPLACE" => Ok(Value::function_replace),

            "NOW" => Ok(Value::function_now),
            "YEAR" => Ok(Value::function_year),
            "MONTH" => Ok(Value::function_month),
            "DAY" => Ok(Value::function_day),
            "HOUR" => Ok(Value::function_hour),
            "DATEADD" => Ok(Value::function_date_add),
            "DATEFROMPARTS" => Ok(Value::function_date_from_parts),

            "ROUND" => Ok(Value::function_round),
            "POW" => Ok(Value::function_pow),

            "IIF" => Ok(Value::function_iif),
            "IFNULL" => Ok(Value::function_if_null),
            "NULLIF" => Ok(Value::function_null_if),

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
