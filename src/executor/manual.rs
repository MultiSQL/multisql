use {
    crate::{
        recipe::{
            Aggregate, BinaryOperator, BooleanCheck, Function, Ingredient, Method, Recipe,
            RecipeError, UnaryOperator,
        },
        Table,
    },
    sqlparser::ast::{Expr, Ident, Query, SelectItem, TableWithJoins},
};

struct Manual {
    initial_table: Table,
    joins: Vec<(Table, (TableOperator, Recipe, Vec<Column>))>,
    selections: Vec<Selection>,
    columns: Vec<Column>,
    groups: Vec<usize>,
    constraint: Recipe,
    contains_aggregate: bool,
}

impl Manual {
    fn write(query: Query) -> Self {
        let mut columns: Vec<(Column, Recipe)> = vec![];

        if let SetExpr::Select(statement) = query {
            if query.len() > 1 {
                panic!("What is this query???");
            }
            let from = query.from[0];
            let initial_table = Table::from(from.relation);
            let joins = from.joins.map(map_join).collect();

            let mut columns = vec![];
            let constraint = recipe(query.selection, columns);
            let selections = query.projection.map(|select_item| {
                let (expression, alias) = match select_item {
                    SelectItem::UnnamedExpr(expression) => (expression, None),
                    SelectItem::ExprWithAlias { expr, alias } => (expr, Some(alias)),
                    _ => unimplemented!(),
                };
                let (contains_aggregate, recipe) = recipe(expression, columns);
                let recipe = recipe.simplify(None);
                Selection { alias, recipe }
            });

            let groups = vec![]; // TODO

            Manual {
                initial_table,
                joins,
                selections,
                columns,
                groups,
                constraint,
                contains_aggregate: false,
            }
        }
    }
}

fn recipe(expression: Expr, &mut columns: Vec<Column>) -> (bool, Recipe) {
    let mut is_aggregate = false;
    macro_rules! aggregate {
        ($aggregate: expr) => {
            is_aggregate = true;
            $aggregate
        };
    }
    match constraint {
        Expr::Identifier(identifier) => {
            let identifier = vec![identifier];
            let index = columns.iter().position(|column| column == identifier);
            Recipe::Ingredient(Ingredient::Column(if let Some(index) = index {
                index
            } else {
                columns.push(identifier);
                columns.len() - 1
            }))
        }
        Expr::CompoundIdentifier(identifier) => {
            let index = columns.iter().position(|column| column == identifier);
            Recipe::Ingredient(Ingredient::Column(if let Some(index) = index {
                index
            } else {
                columns.push(identifier);
                columns.len() - 1
            }))
        } // TODO: Remove duplicate
        Expr::IsNull(expression) => Recipe::Method(Method::BooleanCheck(BooleanCheck::IsNull(
            recipe(expression, columns),
        ))),
        Expr::IsNotNull(expression) => {
            Recipe::Method(Method::UnaryOperator(UnaryOperator::Not(Recipe::Method(
                Method::BooleanCheck(BooleanCheck::IsNull(recipe(expression, columns))),
            ))))
        }
        Expr::Function(function) => {
            let name = function.name;
            let function = Function::from_string(name);
            let function = if function.is_none() {
                let aggregate = Aggregate::from_string(name);
                is_aggregate = aggregate.is_some();
                aggregate
            } else {
                function
            };
            function.map(Ok).unwrap_or(Err(UnimplementedFunction(name)))
        }
        _ => unimplemented!("TODO"),
    }
}

fn map_join(join: TableWithJoins) {
    let contraint = match join.join_operator {
        JoinOperator::Inner(JoinConstraint::On(constraint))
        | JoinOperator::LeftOuter(JoinConstraint::On(constraint))
        | JoinOperator::RightOuter(JoinConstraint::On(constraint))
        | JoinOperator::FullOuter(JoinConstraint::On(constraint)) => constraint,
        //_ => ??
    };
    let mut columns = vec![];
    let constraint = recipe(constraint, columns);
    tables.push((
        Table::from(join.relation),
        (join.join_operator, constraint, columns),
    ));
}

struct Selection {
    alias: Option<Ident>,
    recipe: Recipe,
}

type Column = Vec<Ident>;
