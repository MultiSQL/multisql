use {
    super::join::JoinManual,
    crate::{
        executor::{
            types::{Alias, ObjectName},
            MetaRecipe,
        },
        Result,
    },
    serde::Serialize,
    sqlparser::ast::{Ident, Select, SelectItem as SelectItemAst},
    std::fmt::Debug,
    thiserror::Error as ThisError,
};

#[derive(ThisError, Serialize, Debug, PartialEq)]
pub enum ManualError {
    #[error("this should be impossible, please report")]
    Unreachable,
}

pub struct Manual {
    pub input: ManualInput,
    pub output: ManualOutput,
}
pub struct ManualInput {
    pub joins: Vec<JoinManual>,
    pub select_items: Vec<SelectItem>,
    pub constraint: MetaRecipe,
    pub groups: Vec<MetaRecipe>,
}
pub struct ManualOutput {
    pub select_item_aliases: Vec<Alias>,
}

pub enum SelectItem {
    Recipe(MetaRecipe),
    Wildcard(Option<ObjectName>),
}

impl Manual {
    pub fn new(select: Select) -> Result<Self> {
        let Select {
            projection,
            from,
            selection,
            group_by,
            // TODO (below)
            distinct: _,
            top: _,
            lateral_views: _,
            cluster_by: _,
            distribute_by: _,
            sort_by: _,
            having: _,
        } = select;

        let constraint = selection
            .map(|selection| MetaRecipe::new(selection))
            .unwrap_or(Ok(MetaRecipe::TRUE))?;

        let (select_items, subqueries): (Vec<(SelectItem, Alias)>, Vec<Vec<JoinManual>>) =
            projection
                .into_iter()
                .map(convert_select_item)
                .collect::<Result<Vec<((SelectItem, Alias), Vec<JoinManual>)>>>()?
                .into_iter()
                .unzip();
        let (select_items, select_item_aliases) = select_items.into_iter().unzip();
        let subqueries = subqueries
            .into_iter()
            .reduce(|mut all_subqueries, subqueries| {
                all_subqueries.extend(subqueries);
                all_subqueries
            })
            .ok_or(ManualError::Unreachable)?;

        let mut joins = from
            .into_iter()
            .map(|from| {
                let main = JoinManual::new_implicit_join(from.relation)?;
                let mut joins = from
                    .joins
                    .into_iter()
                    .map(|join| JoinManual::new(join))
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
            .ok_or(ManualError::Unreachable)?;
        joins.extend(subqueries);
        let joins = joins;

        let groups = group_by
            .into_iter()
            .map(|expression| MetaRecipe::new(expression))
            .collect::<Result<Vec<MetaRecipe>>>()?;

        Ok(Manual {
            input: ManualInput {
                joins,
                select_items,
                constraint,
                groups,
            },
            output: ManualOutput {
                select_item_aliases,
            },
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
    select_item: SelectItemAst,
) -> Result<((SelectItem, Alias), Vec<JoinManual>)> {
    Ok(match select_item {
        SelectItemAst::UnnamedExpr(_) | SelectItemAst::ExprWithAlias { .. } => {
            let (expression, alias) = match select_item {
                SelectItemAst::UnnamedExpr(expression) => (expression, None),
                SelectItemAst::ExprWithAlias { expr, alias } => (expr, Some(alias.value)),
                _ => unreachable!(),
            };
            let recipe = MetaRecipe::new(expression)?;
            let subqueries = recipe.meta.subqueries.clone();
            ((SelectItem::Recipe(recipe), alias), subqueries)
        }
        SelectItemAst::Wildcard => ((SelectItem::Wildcard(None), None), vec![]),
        SelectItemAst::QualifiedWildcard(qualifier) => (
            (
                SelectItem::Wildcard(Some(identifier_into_object_name(qualifier.0))),
                None,
            ),
            vec![],
        ),
    })
}
