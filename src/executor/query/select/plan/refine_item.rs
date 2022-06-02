use {
	super::SelectItem,
	crate::{types::ColumnInfo, recipe::PlannedRecipe, Error, PlanError, Result},
};

pub(crate) fn refine_item(
	index: usize,
	select_item: SelectItem,
	columns: &[ColumnInfo],
	include_table: bool,
) -> Result<Vec<(PlannedRecipe, String)>> {
	Ok(match select_item {
		SelectItem::Recipe(meta_recipe, alias) => {
			let recipe = PlannedRecipe::new(meta_recipe, columns)?;
			let label = alias.unwrap_or_else(|| recipe.get_label(index, include_table, &columns));
			vec![(recipe, label)]
		}
		SelectItem::Wildcard(specifier) => {
			let specified_table = specifier.and_then(|specifier| specifier.get(0).cloned());
			let matches_table = |column: &ColumnInfo| {
				specified_table
					.clone()
					.map(|specified_table| {
						column.table.name == specified_table
							|| column
								.table
								.alias
								.clone()
								.map(|alias| alias == specified_table)
								.unwrap_or(false)
					})
					.unwrap_or(true)
			};
			columns
				.iter()
				.enumerate()
				.filter_map(|(index, column)| {
					if matches_table(column) {
						Some((
							PlannedRecipe::of_index(index),
							if include_table {
								format!("{}.{}", column.table.name, column.name)
							} else {
								column.name.clone()
							},
						))
					} else {
						None
					}
				})
				.collect()
		}
	})
}

pub(crate) fn refine_items(
	select_items: Vec<SelectItem>,
	columns: &[ColumnInfo],
	include_table: bool,
) -> Result<Vec<(PlannedRecipe, String)>> {
	select_items
		.into_iter()
		.enumerate()
		.map(|(index, select_item)| refine_item(index, select_item, &columns, include_table))
		.collect::<Result<Vec<Vec<(PlannedRecipe, String)>>>>()?
		.into_iter()
		.reduce(|mut select_items, select_item_set| {
			select_items.extend(select_item_set);
			select_items
		})
		.ok_or(Error::Plan(PlanError::UnreachableNoSelectItems))
}
