use {
	super::join::JoinManual,
	crate::{types::{Alias, ObjectName},
		recipe::{

			MetaRecipe,
		},
		Glue, Result,
	},
	serde::Serialize,
	sqlparser::ast::{Expr, Ident, Select, SelectItem as SelectItemAst},
	std::fmt::Debug,
	thiserror::Error as ThisError,
};

#[derive(ThisError, Serialize, Debug, PartialEq)]
pub enum ManualError {
	#[error("subqueries are not yet supported")]
	UnimplementedSubquery,

	#[error("this should be impossible, please report")]
	UncaughtASTError(String),

	#[error("this should be impossible, please report")]
	Unreachable,
}

pub struct Manual {
	pub joins: Vec<JoinManual>,
	pub select_items: Vec<SelectItem>,
	pub constraint: MetaRecipe,
	pub group_constraint: MetaRecipe,
	pub groups: Vec<MetaRecipe>,
}
pub enum SelectItem {
	Recipe(MetaRecipe, Alias),
	Wildcard(Option<ObjectName>),
}

impl Manual {
	pub fn new(glue: &Glue, select: Select) -> Result<Self> {
		let Select {
			projection,
			from,
			selection,
			group_by,
			having,
			// TODO (below)
			distinct: _,
			top: _,
			lateral_views: _,
			cluster_by: _,
			distribute_by: _,
			sort_by: _,
			into: _,
		} = select;

		let constraint = selection
			.map(|selection| MetaRecipe::new(selection)?.simplify_by_tempdb(&glue.tempdb))
			.unwrap_or(Ok(MetaRecipe::TRUE))?;

		let group_constraint = having
			.map(|having| MetaRecipe::new(having)?.simplify_by_tempdb(&glue.tempdb))
			.unwrap_or(Ok(MetaRecipe::TRUE))?;

		let groups = group_by
			.into_iter()
			.map(|expression| MetaRecipe::new(expression)?.simplify_by_tempdb(&glue.tempdb))
			.collect::<Result<Vec<MetaRecipe>>>()?;

		let (select_items, _subqueries): (Vec<SelectItem>, Vec<Vec<JoinManual>>) = projection
			.into_iter()
			.map(|select_item| convert_select_item(glue, select_item))
			.collect::<Result<Vec<(SelectItem, Vec<JoinManual>)>>>()?
			.into_iter()
			.unzip();

		let joins = from
			.into_iter()
			.map(|from| {
				let main = JoinManual::new_implicit_join(from.relation)?;
				let mut joins = from
					.joins
					.into_iter()
					.map(|join| JoinManual::new(join, &glue.tempdb))
					.collect::<Result<Vec<JoinManual>>>()?;
				joins.push(main);
				Ok(joins)
			})
			.collect::<Result<Vec<Vec<JoinManual>>>>()?
			.into_iter()
			.reduce(|mut all_joins, joins| {
				all_joins.extend(joins);
				all_joins
			})
			.ok_or_else(|| ManualError::UncaughtASTError(String::from("No tables")))?;

		Ok(Manual {
			joins,
			select_items,
			constraint,
			group_constraint,
			groups,
		})
	}
}

fn identifier_into_object_name(identifier: Vec<Ident>) -> ObjectName {
	identifier
		.into_iter()
		.map(|identifier| identifier.value)
		.collect()
}

fn convert_select_item(
	glue: &Glue,
	select_item: SelectItemAst,
) -> Result<(SelectItem, Vec<JoinManual>)> {
	Ok(match select_item {
		SelectItemAst::UnnamedExpr(_) | SelectItemAst::ExprWithAlias { .. } => {
			let (expression, alias) = match select_item {
				SelectItemAst::UnnamedExpr(expression) => {
					let alias = if let Expr::Identifier(identifier) = expression.clone() {
						Some(identifier.value)
					} else {
						None
					};
					(expression, alias)
				}
				SelectItemAst::ExprWithAlias { expr, alias } => (expr, Some(alias.value)),
				_ => unreachable!(),
			};
			let recipe = MetaRecipe::new(expression)?.simplify_by_tempdb(&glue.tempdb)?;
			let subqueries = recipe.meta.subqueries.clone();
			(SelectItem::Recipe(recipe, alias), subqueries)
		}
		SelectItemAst::Wildcard => (SelectItem::Wildcard(None), vec![]),
		SelectItemAst::QualifiedWildcard(qualifier) => (
			SelectItem::Wildcard(Some(identifier_into_object_name(qualifier.0))),
			vec![],
		),
	})
}
