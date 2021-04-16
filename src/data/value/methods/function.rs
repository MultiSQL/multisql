use crate::{Result, Value, ValueError};

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

impl Value {
    pub fn function_if_null(mut arguments: Vec<Self>) -> Result<Self> {
        expect_arguments!(arguments, 2);
        Ok(arguments.remove(0).if_null(arguments.remove(0)))
    }
    pub fn function_iif(mut arguments: Vec<Self>) -> Result<Self> {
        expect_arguments!(arguments, 3);
        arguments
            .remove(0)
            .iif(arguments.remove(0), arguments.remove(0))
    }
    pub fn function_to_lowercase(mut arguments: Vec<Self>) -> Result<Self> {
        expect_arguments!(arguments, 1);
        arguments.remove(0).to_lowercase()
    }
    pub fn function_to_uppercase(mut arguments: Vec<Self>) -> Result<Self> {
        expect_arguments!(arguments, 1);
        arguments.remove(0).to_uppercase()
    }
    pub fn function_left(mut arguments: Vec<Self>) -> Result<Self> {
        expect_arguments!(arguments, 2);
        arguments.remove(0).left(arguments.remove(0))
    }
    pub fn function_right(mut arguments: Vec<Self>) -> Result<Self> {
        expect_arguments!(arguments, 2);
        arguments.remove(0).right(arguments.remove(0))
    }
}
