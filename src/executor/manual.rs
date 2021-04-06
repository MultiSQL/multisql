use {
	crate::{
		BinaryOperator, BooleanCheck, Function, Ingredient, Method, Recipe, Table, UnaryOperator,
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
		recipe = |expression| -> Recipe { recipe };

		if let SetExpr::Select(statement) = query {
			let from = query.from[0]; // >0 = ?????
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
				let recipe = recipe(expression, columns);
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

fn recipe(expression: Expr, &mut columns: Vec<Column>) -> Recipe {
	match constraint {
		Identifier(identifier) => {
			let identifier = vec![identifier];
			let index = columns.iter().position(|column| column == identifier);
			Recipe::Ingredient(Ingredient::Column(if let Some(index) = index {
				index
			} else {
				columns.push(identifier);
				columns.len() - 1
			}))
		}
		CompoundIdentifier(identifier) => {
			let index = columns.iter().position(|column| column == identifier);
			Recipe::Ingredient(Ingredient::Column(if let Some(index) = index {
				index
			} else {
				columns.push(identifier);
				columns.len() - 1
			}))
		} // TODO: Remove duplicate
		IsNull(expression) => Recipe::Method(Method::BooleanCheck(BooleanCheck::IsNull(recipe(
			expression, columns,
		)))),
		IsNotNull(expression) => {
			Recipe::Method(Method::UnaryOperator(UnaryOperator::Not(Recipe::Method(
				Method::BooleanCheck(BooleanCheck::IsNull(recipe(expression, columns))),
			))))
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
