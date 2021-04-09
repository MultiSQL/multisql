use {
    super::{manual::Selection, Find, Method, Recipe},
    crate::Result,
};

#[derive(Debug, Clone)]
pub struct Subquery {
    pub table: String,
    pub column: Recipe,
    pub constraint: Recipe,
}

pub struct MacroComponents {
    pub aggregate_selection_indexes: Vec<usize>,
    pub subqueries: Vec<Subquery>,
}
impl MacroComponents {
    pub fn new(selections: &Vec<Selection>) -> Result<MacroComponents> {
        let selections: Vec<&Recipe> = selections
            .into_iter()
            .filter_map(|selection| match selection {
                Selection::Recipe { recipe, .. } => Some(recipe),
                _ => None,
            })
            .collect();
        let aggregate_selection_indexes = selections
            .iter()
            .enumerate()
            .filter_map(|(index, &selection)| {
                selection
                    .clone()
                    .contains(|method| matches!(method, Method::Aggregate(..)))
                    .then(|| index)
            })
            .collect();
        let subqueries = selections
            .iter()
            .fold(vec![], |mut subqueries, &selection| {
                selection
                    .clone()
                    .get(|method| matches!(method, Method::Subquery(..)))
                    .map(|selection_subqueries| {
                        subqueries.extend(
                            selection_subqueries
                                .into_iter()
                                .map(|subquery| {
                                    if let Method::Subquery(subquery) = subquery {
                                        subquery.clone()
                                    } else {
                                        unreachable!(
                                            "reportable: fault macro components, subqueries get"
                                        )
                                    }
                                })
                                .collect::<Vec<Subquery>>(),
                        );
                    });
                subqueries
            });
        Ok(MacroComponents {
            aggregate_selection_indexes,
            subqueries,
        })
    }
}
