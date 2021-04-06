use {
    super::{
        Aggregate, BinaryOperator, BooleanCheck, Function, Ingredient, Method, Recipe, RecipeError,
        Resolve, UnaryOperator,
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
    pub groups: Vec<usize>,
    pub constraint: Recipe,
    pub contains_aggregate: bool,
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
                panic!("What is this query???");
            }
            let from = statement.from.get(0).unwrap(); // TODO: Handle!
            let initial_table = table_identity(Table::new(&from.relation).unwrap()); // TODO: Handle!
            let joins = from.joins.clone().into_iter().map(map_join).collect();

            let mut needed_columns = vec![];
            let mut contains_aggregate = false;
            let constraint = statement
                .selection
                .map(|selection| {
                    let (recipe, _) = recipe(selection, &mut needed_columns).unwrap(); /* TODO: Handle! */
                    recipe
                })
                .unwrap_or(Recipe::Ingredient(Ingredient::Value(Value::Null)));
            let selections = statement
                .projection
                .into_iter()
                .map(|select_item| {
                    match select_item {
                        SelectItem::UnnamedExpr(_) | SelectItem::ExprWithAlias { .. } => {
                            let (expression, alias) = match select_item {
                                SelectItem::UnnamedExpr(expression) => (expression, None),
                                SelectItem::ExprWithAlias { expr, alias } => (expr, Some(alias)),
                                _ => unreachable!(),
                            };
                            let recipe = recipe_set_aggregate(
                                expression,
                                &mut needed_columns,
                                &mut contains_aggregate,
                            )
                            .unwrap(); // TODO: Handle!
                            let recipe = recipe.simplify(None).unwrap(); // TODO: Handle!
                            Selection::Recipe { alias, recipe }
                        }
                        SelectItem::Wildcard => Selection::Wildcard { qualifier: None },
                        SelectItem::QualifiedWildcard(qualifier) => Selection::Wildcard {
                            qualifier: Some(qualifier.0),
                        },
                    }
                })
                .collect();

            let groups = vec![]; // TODO

            Ok(Manual {
                initial_table,
                joins,
                selections,
                needed_columns,
                groups,
                constraint,
                contains_aggregate: false,
            })
        } else {
            unimplemented!()
        }
    }
}

fn convert_join(from: JoinOperatorAst) -> (JoinOperator, Recipe, Vec<ObjectName>) {
    let mut columns = vec![];
    let values = match from {
        JoinOperatorAst::Inner(JoinConstraint::On(constraint)) => {
            (JoinOperator::Inner, Some(constraint))
        }
        _ => unreachable!(),
    };
    (
        values.0,
        recipe_no_aggregate(values.1.unwrap(), &mut columns).unwrap(), /*TODO: Handle*/
        columns,
    )
}

fn table_identity(table: Table) -> TableIdentity {
    (table.get_alias().clone(), table.get_name().clone())
}

fn recipe(expression: Expr, columns: &mut Vec<ObjectName>) -> Result<(Recipe, bool)> {
    let mut is_aggregate = false;
    let is_aggregate_ref = &mut is_aggregate;

    let recipe = match expression {
        Expr::Identifier(identifier) => Ok(column_recipe(vec![identifier], columns)),
        Expr::CompoundIdentifier(identifier) => Ok(column_recipe(identifier, columns)),
        Expr::Value(value) => Ok(Recipe::Ingredient(Ingredient::Value(Value::try_from(
            Literal::try_from(&value)?,
        )?))),
        Expr::IsNull(expression) => Ok(Recipe::Method(Box::new(Method::BooleanCheck(
            BooleanCheck::IsNull,
            recipe_set_aggregate(*expression, columns, is_aggregate_ref)?,
        )))),
        Expr::IsNotNull(expression) => Ok(Recipe::Method(Box::new(Method::UnaryOperation(
            UnaryOperator::Not,
            Recipe::Method(Box::new(Method::BooleanCheck(
                BooleanCheck::IsNull,
                recipe_set_aggregate(*expression, columns, is_aggregate_ref)?,
            ))),
        )))),
        Expr::UnaryOp { op, expr } => Ok(Recipe::Method(Box::new(Method::UnaryOperation(
            op.try_into()?,
            recipe_set_aggregate(*expr, columns, is_aggregate_ref)?,
        )))),
        Expr::BinaryOp { op, left, right } => {
            Ok(Recipe::Method(Box::new(Method::BinaryOperation(
                op.try_into()?,
                recipe_set_aggregate(*left, columns, is_aggregate_ref)?,
                recipe_set_aggregate(*right, columns, is_aggregate_ref)?,
            ))))
        }
        Expr::Function(function) => {
            let name = function.name.0[0].value.clone();
            let mut arguments = vec![];
            for result in function
                .args
                .into_iter()
                .map(|argument| recipe_from_argument(argument, columns))
            {
                let (recipe, aggregate) = result?;
                if !is_aggregate && aggregate {
                    is_aggregate = true;
                }
                arguments.push(recipe);
            } // TODO: Improve
            let function: Result<Function> = name.clone().try_into();
            if function.is_ok() {
                Ok(Recipe::Method(Box::new(Method::Function(
                    function?, arguments,
                ))))
            } else {
                let aggregate: Result<Aggregate> = name.try_into();
                is_aggregate = aggregate.is_ok();
                Ok(Recipe::Method(Box::new(Method::Aggregate(
                    aggregate?,
                    arguments.get(0).unwrap().clone(), /* TODO: Handle! */
                ))))
            }
        }
        Expr::Nested(expression) => recipe(*expression, columns).map(|(recipe, aggregate)| {
            is_aggregate = aggregate;
            recipe
        }),
        unimplemented => Err(RecipeError::UnimplementedExpression(unimplemented).into()),
    };
    recipe.map(|recipe| (recipe, is_aggregate))
}

fn recipe_no_aggregate(expression: Expr, columns: &mut Vec<ObjectName>) -> Result<Recipe> {
    recipe(expression, columns).map(|(recipe, _)| recipe)
}

fn recipe_set_aggregate(
    expression: Expr,
    columns: &mut Vec<ObjectName>,
    is_aggregate: &mut bool,
) -> Result<Recipe> {
    let (recipe, aggregate) = recipe(expression, columns)?;
    if !*is_aggregate && aggregate {
        *is_aggregate = true; // TODO: !!!!: I suspect this will not work
    }
    Ok(recipe)
}

fn recipe_from_argument(
    argument: FunctionArg,
    columns: &mut Vec<ObjectName>,
) -> Result<(Recipe, bool)> {
    match argument {
        FunctionArg::Named { arg, .. } | FunctionArg::Unnamed(arg) => recipe(arg, columns),
    }
}

fn map_join(join: AstJoin) -> Join {
    (
        table_identity(Table::new(&join.relation).unwrap()), // TODO: Handle
        convert_join(join.join_operator),
    )
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
