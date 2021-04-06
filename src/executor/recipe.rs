use {
    crate::{Row, Value},
    sqlparser::DataType,
    std::cmp::min,
};

enum Recipe {
    Ingredient(Ingredient),
    Method(Method),
}

enum Ingredient {
    Value(Value),
    Column(usize),
}

enum Method {
    Value(Value), // SIMPLIFICATION ONLY!

    BooleanCheck(BooleanCheck, Recipe),
    UnaryOperation(UnaryOperator, Recipe),
    BinaryOperation(BinaryOperator, Recipe, Recipe),
    Function(Function, Vec<Recipe>),

    Cast(DataType, Recipe),

    Aggregate(Aggregate, Recipe),
}

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

use {serde::Serialize, std::fmt::Debug, thiserror::Error};
#[derive(Error, Serialize, Debug, PartialEq)]
enum RecipeError {
    #[error("recipe missing components")]
    MissingComponents,

    #[error("{0} is either invalid or unimplemented")]
    UnimplementedFunction(String),
}

type RecipeKey = Option<Row>;
type RecipeSolution = Option<Result<Value>>;

trait Resolve {
    fn solve(self, row: RecipeKey) -> RecipeSolution;
    fn simplify(self, row: RecipeKey) -> Result<Self>;
}

impl Recipe {
    fn as_solution(self, row: RecipeKey) -> Option<Value> {
        if let Recipe::Ingredient(Ingredient::Value(value)) = self {
            Some(value)
        } else {
            None
        }
    }
    fn must_solve(self, row: RecipeKey) -> Result<Value> {
        self.solve(row).or(Err(MissingComponents.into()))
    }
}

impl Resolve for Recipe {
    fn solve(self, row: RecipeKey) -> RecipeSolution {
        match self {
            Recipe::Ingredient(ingredient) => ingredient.solve(row),
            Recipe::Method(method) => method.solve(row),
        }
    }
    fn simplify(self, row: RecipeKey) -> Result<Self> {
        match self {
            Recipe::Ingredient(ingredient) => Recipe::Ingredient(ingredient.simplify(row)),
            Recipe::Method(method) => method.simplify(row).map(|method| {
                if let Method::Value(value) = method {
                    Recipe::Ingredient(Ingredient::Value(value))
                } else {
                    Recipe::Method(method)
                }
            }),
        }
    }
}

impl Resolve for Ingredient {
    fn solve(self, row: RecipeKey) -> RecipeSolution {
        match self {
            Ingredient::Value(value) => value,
            Ingredient::Column(index) => row.map(row.get(index)),
        }
    }
    fn simplify(self, row: RecipeKey) -> Result<Self> {
        self.solve(row)
            .map(|result| Ingredient::Value(result?))
            .or(self);
    }
}

impl Resolve for Method {
    fn solve(self, row: RecipeKey) -> RecipeSolution {
        match self {
            Method::BooleanCheck(check, recipe) => check.solve(recipe.solve(row)??),
            Method::UnaryOperation(operator, recipe) => operator.solve(recipe.solve(row)??),
            Method::BinaryOperation(operator, left, right) => {
                operator.solve(left.solve(row)??, right.solve(row)??)
            }
            Method::Function(function, arguments) => {
                let arguments = arguments.into_iter().map(|argument| argument.solve(row));
                if let Some(issue) =
                    arguments.find(|argument| matches!(argument, None | Some(Err(_))))
                {
                    issue
                } else {
                    function(function, arguments.collect())
                }
            }
            _ => unimplemented!(),
        }
    }
    fn simplify(self, row: RecipeKey) -> Result<Self> {
        match self {
            Method::Aggregate(aggregate, recipe) => Method::Aggregate(aggregate, recipe.simplify()),
            Method::BinaryOperation(operator, left, right) => {
                let (left, right) = (left.simplify(row)?, right.simplify(row)?);
                if let (Some(left), Some(right)) = (left.as_solution(), right.as_solution()) {
                    Method::Value(operator.solve(left, right))
                } else {
                    Method::BinaryOperation(operator, left, right)
                }
            }
            _ => self,
        }
    }
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

trait FromString {
    fn from_string(string: String) -> Option<Self>;
}

impl FromString for Function {
    fn from_string(string: String) -> Option<Self> {
        match string.as_uppercase() {
            "UPPER" => Some(Function::Upper),
            "LOWER" => Some(Function::Lower),
            "LEFT" => Some(Function::Left),
            "RIGHT" => Some(Function::Right),
            _ => None,
        }
    }
}

impl FromString for Aggregate {
    fn from_string(string: String) -> Option<Self> {
        match string.as_uppercase() {
            "MIN" => Some(Aggregate::Min),
            "MAX" => Some(Aggregate::Max),
            "SUM" => Some(Aggregate::Sum),
            "AVG" => Some(Aggregate::Avg),
            _ => None,
        }
    }
}
