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
macro_rules! of_parts {
    ($first_part: ident, $($part: ident),+ $final_part: ident, $value: expr) => {
        $first_part$(::$part($part)+::$final_part($expr)$())+
    };
}

fn recipe(expression: Expr, &mut columns: Vec<Column>) -> Result<(Recipe, bool)> {
    let mut is_aggregate = false;

    let recipe = match expression {
        Expr::Identifier(identifier) => Ok(column_recipe(vec![identifier], columns)),
        Expr::CompoundIdentifier(identifier) => Ok(column_recipe(identifier, columns)),
        Expr::IsNull(expression) => Ok(of_parts(
            Recipe,
            Method,
            BooleanCheck,
            IsNull,
            recipe(expression, columns),
        )),
        Expr::IsNotNull(expression) => Ok(of_parts(
            Recipe,
            Method,
            UnaryOperator,
            Not,
            of_parts(
                Recipe,
                Method,
                BooleanCheck,
                IsNull,
                recipe(expression, columns),
            ),
        )),
        Expr::UnaryOp { op, expr } => Method::UnaryOperation(op.try_into()?, expr),
        Expr::BinaryOp { op, left, right } => Method::BinaryOperation(op.try_into()?, left, right),
        Expr::Function(function) => {
            let function = Function::from_string(function.name);
            if function.is_ok() {
                function
            } else {
                let aggregate = Aggregate::from_string(name);
                is_aggregate = aggregate.is_ok();
                aggregate
            }
        }
        Expr::Nested(expression) => recipe(expression, columns).map(|(recipe, aggregate)| {
            is_aggregate = aggregate;
            recipe
        }),
        unimplemented => Err(UnimplementedExpression(unimplemented).into()),
    };
    recipe.map(|recipe| (recipe, is_aggregate))
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

fn column_recipe(identifier: Vec<Ident>, &mut columns: Vec<Column>) -> Recipe {
    let index = columns.iter().position(|column| column == identifier);
    let index = if let Some(index) = index {
        index
    } else {
        columns.push(identifier);
        columns.len() - 1
    };
    Recipe::Ingredient(Ingredient::Column(index))
}

struct Selection {
    alias: Option<Ident>,
    recipe: Recipe,
}

type Column = Vec<Ident>;
