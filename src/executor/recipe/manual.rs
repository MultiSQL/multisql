use {
    super::{
        Aggregate, BinaryOperator, BooleanCheck, Function, Ingredient, MacroComponents, Method,
        Recipe, RecipeError, Resolve, Subquery, UnaryOperator, RECIPE_NULL,
    },
    crate::{Literal, Result, Row, Table, Value},
    sqlparser::ast::{
        Expr, FunctionArg, Ident, Join as AstJoin, JoinConstraint, JoinOperator as JoinOperatorAst,
        Query, SelectItem, SetExpr,
    },
    std::convert::{TryFrom, TryInto},
};

pub type ObjectName = Vec<Ident>;
pub type Join = (TableIdentity, (JoinOperator, Recipe, Vec<ObjectName>));
pub type TableIdentity = (String /*alias*/, String /*name*/);
pub type ColumnsAndRows = (Vec<ObjectName>, Vec<Row>);
pub type LabelsAndRows = (Vec<String>, Vec<Row>);
pub type LabelledSelection = (Recipe, String);

pub struct Manual {
    pub initial_table: TableIdentity,
    pub joins: Vec<Join>,
    pub selections: Vec<Selection>,
    pub needed_columns: Vec<ObjectName>,
    pub groups: Vec<Recipe>,
    pub constraint: Recipe,
    pub aggregate_selection_indexes: Vec<usize>,
    pub limit: Recipe,
}

pub enum Selection {
    Recipe {
        alias: Option<Ident>,
        recipe: Recipe,
    },
    Wildcard {
        qualifier: Option<ObjectName>,
    },
}

pub enum JoinOperator {
    Inner,
    Left,
    Right,
    Full,
}

impl Manual {
    pub fn write(query: Query) -> Result<Self> {
        if let SetExpr::Select(statement) = query.body {
            if statement.from.len() > 1 {
                return Err(RecipeError::UnimplementedQuery(format!("{:?}", statement)).into());
            }

            let from = statement
                .from
                .get(0)
                .ok_or(RecipeError::InvalidQuery(format!("{:?}", statement)))?;
            let initial_table = table_identity(Table::new(&from.relation)?);

            let mut needed_columns = vec![];
            let constraint = statement
                .selection
                .map(|selection| recipe(selection, &mut needed_columns))
                .unwrap_or(Ok(Recipe::Ingredient(Ingredient::Value(Value::Null))))?;
            let limit = query
                .limit
                .or(statement.top.map(|top| top.quantity).flatten())
                .map(|expression| recipe(expression, &mut needed_columns))
                .unwrap_or(Ok(RECIPE_NULL))?;

            let selections = statement
                .projection
                .into_iter()
                .map(|select_item| {
                    Ok(match select_item {
                        SelectItem::UnnamedExpr(_) | SelectItem::ExprWithAlias { .. } => {
                            let (expression, alias) = match select_item {
                                SelectItem::UnnamedExpr(expression) => (expression, None),
                                SelectItem::ExprWithAlias { expr, alias } => (expr, Some(alias)),
                                _ => unreachable!(),
                            };
                            let recipe = recipe(expression, &mut needed_columns)?;
                            Selection::Recipe { alias, recipe }
                        }
                        SelectItem::Wildcard => Selection::Wildcard { qualifier: None },
                        SelectItem::QualifiedWildcard(qualifier) => Selection::Wildcard {
                            qualifier: Some(qualifier.0),
                        },
                    })
                })
                .collect::<Result<Vec<Selection>>>()?;

            let joins = from
                .joins
                .clone()
                .into_iter()
                .map(map_join)
                .collect::<Result<Vec<Join>>>()?;

            let groups = statement
                .group_by
                .into_iter()
                .map(|expression| recipe(expression, &mut needed_columns))
                .collect::<Result<Vec<Recipe>>>()?;

            let MacroComponents {
                aggregate_selection_indexes,
                subqueries,
            } = MacroComponents::new(&selections)?;

            let mut joins = joins;
            joins.extend(
                subqueries
                    .into_iter()
                    .map(map_subquery_to_join)
                    .collect::<Vec<Join>>(),
            );
            let joins = joins;

            Ok(Manual {
                initial_table,
                joins,
                selections,
                needed_columns,
                groups,
                constraint,
                aggregate_selection_indexes,
                limit,
            })
        } else {
            Err(RecipeError::UnimplementedQuery(format!("{:?}", query)).into())
        }
    }
}

fn map_subquery_to_join(subquery: Subquery) -> Join {
    (
        (String::new() /* Alias */, subquery.table),
        (
            JoinOperator::Inner, /* TODO: Think about join type*/
            subquery.constraint,
            vec![],
        ),
    )
}

fn convert_join(from: JoinOperatorAst) -> Result<(JoinOperator, Recipe, Vec<ObjectName>)> {
    let mut columns = vec![];
    let values = match from {
        JoinOperatorAst::Inner(JoinConstraint::On(constraint)) => (JoinOperator::Inner, constraint),
        _ => unreachable!(),
    };
    Ok((values.0, recipe(values.1, &mut columns)?, columns))
}

fn table_identity(table: Table) -> TableIdentity {
    (table.get_alias().clone(), table.get_name().clone())
}

fn recipe(expression: Expr, columns: &mut Vec<ObjectName>) -> Result<Recipe> {
    match expression {
        Expr::Identifier(identifier) => {
            #[cfg(feature = "double_quote_strings")]
            if identifier.quote_style == Some('"') {
                Ok(Recipe::Ingredient(Ingredient::Value(Value::Str(
                    identifier.value,
                ))))
            } else {
                Ok(column_recipe(vec![identifier], columns))
            }
            #[cfg(not(feature = "double_quote_strings"))]
            Ok(column_recipe(vec![identifier], columns))
        }
        Expr::CompoundIdentifier(identifier) => Ok(column_recipe(identifier, columns)),
        Expr::Value(value) => Ok(Recipe::Ingredient(Ingredient::Value(Value::try_from(
            Literal::try_from(&value)?,
        )?))),
        Expr::IsNull(expression) => Ok(Recipe::Method(Box::new(Method::BooleanCheck(
            BooleanCheck::IsNull,
            recipe(*expression, columns)?,
        )))),
        Expr::IsNotNull(expression) => Ok(Recipe::Method(Box::new(Method::UnaryOperation(
            UnaryOperator::Not,
            Recipe::Method(Box::new(Method::BooleanCheck(
                BooleanCheck::IsNull,
                recipe(*expression, columns)?,
            ))),
        )))),
        Expr::UnaryOp { op, expr } => Ok(Recipe::Method(Box::new(Method::UnaryOperation(
            op.try_into()?,
            recipe(*expr, columns)?,
        )))),
        Expr::BinaryOp { op, left, right } => {
            Ok(Recipe::Method(Box::new(Method::BinaryOperation(
                op.try_into()?,
                recipe(*left, columns)?,
                recipe(*right, columns)?,
            ))))
        }
        Expr::Function(function) => {
            let name = function.name.0[0].value.clone();
            let arguments = function
                .args
                .into_iter()
                .map(|argument| recipe_from_argument(argument, columns))
                .collect::<Result<Vec<Recipe>>>()?;
            let function: Result<Function> = name.clone().try_into();
            if function.is_ok() {
                Ok(Recipe::Method(Box::new(Method::Function(
                    function?, arguments,
                ))))
            } else {
                Ok(Recipe::Method(Box::new(Method::Aggregate(
                    name.try_into()?,
                    arguments
                        .get(0)
                        .ok_or(RecipeError::InvalidFunction)?
                        .clone(),
                ))))
            }
        }
        Expr::Subquery(query) => {
            if let SetExpr::Select(statement) = query.body {
                let table = statement
                    .from
                    .get(0)
                    .ok_or(RecipeError::InvalidQuery(format!("{:?}", statement)))?
                    .relation
                    .clone();
                let table = table_identity(Table::new(&table)?).1;

                let column = statement
                    .projection
                    .get(0)
                    .map(|item| {
                        if let SelectItem::UnnamedExpr(expression) = item {
                            Some(recipe(expression.clone(), columns))
                        } else {
                            None
                        }
                    })
                    .flatten()
                    .ok_or(RecipeError::InvalidQuery(format!("{:?}", statement)))??;

                Ok(Recipe::Method(Box::new(Method::Subquery(Subquery {
                    table,
                    column,
                    constraint: statement
                        .selection
                        .map::<Result<Recipe>, _>(|selection| recipe(selection, columns))
                        .unwrap_or(Ok(Recipe::Ingredient(Ingredient::Value(Value::Null))))?,
                }))))
            } else {
                Err(RecipeError::UnimplementedQuery(format!("{:?}", query)).into())
            }
        }
        Expr::Nested(expression) => recipe(*expression, columns),
        unimplemented => Err(RecipeError::UnimplementedExpression(unimplemented).into()),
    }
}

fn recipe_from_argument(argument: FunctionArg, columns: &mut Vec<ObjectName>) -> Result<Recipe> {
    match argument {
        FunctionArg::Named { arg, .. } | FunctionArg::Unnamed(arg) => recipe(arg, columns),
    }
}

fn map_join(join: AstJoin) -> Result<Join> {
    Ok((
        table_identity(Table::new(&join.relation)?),
        convert_join(join.join_operator)?,
    ))
}

pub fn column_recipe(identifier: ObjectName, columns: &mut Vec<ObjectName>) -> Recipe {
    let index = columns.into_iter().position(|column| column == &identifier);
    let index = if let Some(index) = index {
        index
    } else {
        columns.push(identifier);
        columns.len() - 1
    };
    Recipe::Ingredient(Ingredient::Column(index))
}
