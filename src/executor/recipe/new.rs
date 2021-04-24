use {
    super::{Ingredient, Method, Recipe, RecipeError, TryIntoMethod},
    crate::{
        executor::{
            query::{JoinManual, JoinType},
            types::{ComplexTableName, ObjectName},
        },
        Result, Value,
    },
    sqlparser::ast::{Expr, FunctionArg, Ident, SelectItem, SetExpr},
    std::convert::TryFrom,
};

#[derive(Debug, Clone)]
pub struct MetaRecipe {
    pub recipe: Recipe,
    pub meta: RecipeMeta,
}
impl MetaRecipe {
    pub fn new(expression: Expr) -> Result<Self> {
        let (recipe, meta) = Recipe::new_with_meta(expression)?;
        Ok(Self { recipe, meta })
    }
}
impl MetaRecipe {
    pub const NULL: Self = MetaRecipe {
        recipe: Recipe::Ingredient(Ingredient::Value(Value::Null)),
        meta: RecipeMeta::NEW,
    };
    pub const TRUE: Self = MetaRecipe {
        recipe: Recipe::Ingredient(Ingredient::Value(Value::Bool(true))),
        meta: RecipeMeta::NEW,
    };
}

#[derive(Debug, Clone)]
pub struct RecipeMeta {
    pub columns: Vec<ObjectName>,
    pub aggregates: Vec<Recipe>,
    pub subqueries: Vec<JoinManual>,
}
impl RecipeMeta {
    pub const NEW: Self = Self {
        columns: vec![],
        aggregates: vec![],
        subqueries: vec![],
    };
    fn append_column(&mut self, column: ObjectName) {
        self.columns.push(column);
    }
    fn append_aggregate(&mut self, aggregate: Recipe) {
        self.aggregates.push(aggregate);
    }
    fn append_subquery(&mut self, subquery: JoinManual) {
        self.subqueries.push(subquery);
    }
    fn find_column(&self, column: &ObjectName) -> Option<usize> {
        self.columns
            .iter()
            .position(|search_column| search_column == column)
    }
    pub fn find_or_append_column(&mut self, column: ObjectName) -> usize {
        self.find_column(&column).unwrap_or({
            self.append_column(column);
            self.columns.len() - 1
        })
    }
    pub fn aggregate(&mut self, aggregate: Recipe) -> Recipe {
        self.append_aggregate(aggregate);
        let index = self.aggregates.len() - 1;
        Recipe::Ingredient(Ingredient::Aggregate(index))
    }
    pub fn subquery(&mut self, subquery: Subquery) -> Result<Recipe> {
        let result = subquery.column;
        let table = subquery.table;
        let join_type = JoinType::Left;
        let constraint = subquery
            .constraint
            .map(MetaRecipe::new)
            .unwrap_or(Ok(MetaRecipe::NULL))?;
        let subquery = JoinManual {
            table,
            join_type,
            constraint,
        };
        self.append_subquery(subquery);
        Ok(result)
    }
    pub fn aggregate_average(&mut self, argument: Recipe) -> Recipe {
        Recipe::Method(Box::new(Method::BinaryOperation(
            Value::generic_divide,
            self.aggregate(Recipe::Method(Box::new(Method::Aggregate(
                Value::aggregate_sum,
                argument.clone(),
            )))),
            self.aggregate(Recipe::Method(Box::new(Method::Aggregate(
                Value::aggregate_count,
                argument,
            )))),
        )))
    }
}

pub struct Subquery {
    pub table: ComplexTableName,
    pub column: Recipe,
    pub constraint: Option<Expr>,
}

impl Recipe {
    pub fn new_without_meta(expression: Expr) -> Result<Self> {
        Self::new_with_meta(expression).map(|(new, _)| new)
    }
    fn new_with_meta(expression: Expr) -> Result<(Self, RecipeMeta)> {
        let mut meta = RecipeMeta::NEW;
        Ok((Self::with_meta(expression, &mut meta)?, meta))
    }
    fn with_meta(expression: Expr, meta: &mut RecipeMeta) -> Result<Self> {
        let error_expression_clone = expression.clone();
        match expression {
            Expr::Identifier(identifier) => {
                #[cfg(feature = "double_quote_strings")]
                if identifier.quote_style == Some('"') {
                    Ok(Recipe::Ingredient(Ingredient::Value(Value::Str(
                        identifier.value,
                    ))))
                } else {
                    Ok(Self::from_column(
                        identifier_into_object_name(vec![identifier]),
                        meta,
                    ))
                }
                #[cfg(not(feature = "double_quote_strings"))]
                Ok(Self::from_column(
                    identifier_into_object_name(vec![identifier]),
                    meta,
                ))
            }
            Expr::CompoundIdentifier(identifier) => Ok(Self::from_column(
                identifier_into_object_name(identifier),
                meta,
            )),
            Expr::Value(value) => Ok(Recipe::Ingredient(Ingredient::Value(Value::try_from(
                &value,
            )?))),
            Expr::IsNull(expression) => Ok(Recipe::Method(Box::new(Method::UnaryOperation(
                Value::is_null,
                Self::with_meta(*expression, meta)?,
            )))),
            Expr::IsNotNull(expression) => Ok(Recipe::Method(Box::new(Method::UnaryOperation(
                Value::not,
                Recipe::Method(Box::new(Method::UnaryOperation(
                    Value::is_null,
                    Self::with_meta(*expression, meta)?,
                ))),
            )))),
            Expr::UnaryOp { op, expr } => Ok(Recipe::Method(Box::new(Method::UnaryOperation(
                op.into_method()?,
                Self::with_meta(*expr, meta)?,
            )))),
            Expr::BinaryOp { op, left, right } => {
                Ok(Recipe::Method(Box::new(Method::BinaryOperation(
                    op.into_method()?,
                    Self::with_meta(*left, meta)?,
                    Self::with_meta(*right, meta)?,
                ))))
            }
            Expr::Function(function) => {
                let name = function.name.0[0].value.clone();
                if name == "AVG" {
                    let argument = function
                        .args
                        .get(0)
                        .ok_or(RecipeError::InvalidExpression(error_expression_clone))?
                        .clone();
                    let argument = Recipe::from_argument(argument, meta)?;

                    Ok(meta.aggregate_average(argument))
                } else if let Ok(function_operator) = name.clone().into_method() {
                    let arguments = function
                        .args
                        .into_iter()
                        .map(|argument| Recipe::from_argument(argument, meta))
                        .collect::<Result<Vec<Recipe>>>()?;
                    Ok(Recipe::Method(Box::new(Method::Function(
                        function_operator,
                        arguments,
                    ))))
                } else {
                    let argument = function
                        .args
                        .get(0)
                        .ok_or(RecipeError::InvalidExpression(error_expression_clone))?
                        .clone();
                    let argument = Recipe::from_argument(argument, meta)?;

                    Ok(meta.aggregate(Recipe::Method(Box::new(Method::Aggregate(
                        name.into_method()?,
                        argument,
                    )))))
                }
            }
            Expr::Cast { data_type, expr } => Ok(Recipe::Method(Box::new(Method::Cast(
                data_type,
                Self::with_meta(*expr, meta)?,
            )))),
            Expr::Between {
                negated,
                expr,
                low,
                high,
            } => {
                let body = Method::BinaryOperation(
                    Value::and,
                    Recipe::Method(Box::new(Method::BinaryOperation(
                        Value::gt_eq,
                        Self::with_meta(*expr.clone(), meta)?,
                        Self::with_meta(*low, meta)?,
                    ))),
                    Recipe::Method(Box::new(Method::BinaryOperation(
                        Value::lt_eq,
                        Self::with_meta(*expr, meta)?,
                        Self::with_meta(*high, meta)?,
                    ))),
                );
                let body = if negated {
                    Method::UnaryOperation(Value::not, Recipe::Method(Box::new(body)))
                } else {
                    body
                };
                Ok(Recipe::Method(Box::new(body)))
            }
            Expr::Subquery(query) => {
                if let SetExpr::Select(statement) = query.body {
                    let table = statement
                        .from
                        .get(0)
                        .ok_or(RecipeError::InvalidQuery(format!("{:?}", statement)))?
                        .relation
                        .clone();
                    let table = JoinManual::table_identity(table)?;

                    let column = statement
                        .projection
                        .get(0)
                        .map(|item| {
                            if let SelectItem::UnnamedExpr(expression) = item {
                                Some(Self::with_meta(expression.clone(), meta))
                            } else {
                                None
                            }
                        })
                        .flatten()
                        .ok_or(RecipeError::InvalidQuery(format!("{:?}", statement)))??;

                    let constraint = statement.selection;

                    Ok(meta.subquery(Subquery {
                        table,
                        column,
                        constraint,
                    })?)
                } else {
                    Err(RecipeError::UnimplementedQuery(format!("{:?}", query)).into())
                }
            }
            Expr::Nested(expression) => Self::with_meta(*expression, meta),
            unimplemented => Err(RecipeError::UnimplementedExpression(unimplemented).into()),
        }
    }
    fn from_argument(argument: FunctionArg, meta: &mut RecipeMeta) -> Result<Recipe> {
        match argument {
            FunctionArg::Named { arg, .. } | FunctionArg::Unnamed(arg) => {
                Self::with_meta(arg, meta)
            }
        }
    }
    fn from_column(column: ObjectName, meta: &mut RecipeMeta) -> Recipe {
        Recipe::Ingredient(Ingredient::Column(meta.find_or_append_column(column)))
    }
}

fn identifier_into_object_name(identifier: Vec<Ident>) -> ObjectName {
    identifier
        .into_iter()
        .map(|identifier| identifier.value)
        .collect()
}
