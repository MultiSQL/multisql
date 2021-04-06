use crate::{Manual, Result};

pub async fn select<'a, T: 'static + Debug>(
	storage: &'a dyn Store<T>,
	query: &'a Query,
) -> Result<(Vec<String> /* Labels */, Vec<Row>)> {
	let manual = Manual::write(query);

	let initial_table_name = manual.initial_table.get_name();
	let columns = fetch_columns(storage, initial_table_name).await?;
	let rows = storage.scan_data(initial_table_name).await?;

	let (columns, rows) = manual.joins.into_iter().fold(
		(columns, rows),
		|(columns, rows), (table, (join_operation, recipe, needed_columns))| {
			let table_name = table.get_name();
			let join_columns = fetch_columns(storage, table_name).await?;
			let join_rows = storage.scan_data(table_name).await?;

			let rows = rows.into_iter().fold(vec![].into_iter(), |rows, row| {
				rows.chain(
					join_rows
						.into_iter()
						.map(|join_row| row.0.append(join_row.0)),
				)
			});

			let columns = columns.append(join_columns);

			let needed_column_indexes = needed_columns
				.map(|needed_column| {
					columns
						.iter()
						.enumerate()
						.filter(|(index, column)| needed_column == column)
						.map(|(index, _)| index)
				})
				.collect();

			let check_rows = rows.map(|row| Row(needed_column_indexes.map(|index| row.get(index))));

			let rows = rows
				.enumerate()
				.filter(|(index, row)| {
					recipe.solve(check_rows[index]).unwrap_or(Value::Null) == Value::Bool(true)
				})
				.map(|(_, row)| row);

			// Only inner for now

			(columns, rows)
		},
	);

	// TODO: Constraint

	let needed_column_indexes = manual
		.columns
		.map(|needed_column| {
			columns
				.iter()
				.enumerate()
				.filter(|(index, column)| needed_column == column)
				.map(|(index, _)| index)
		})
		.collect();

	let rows = rows.map(|row| Row(needed_column_indexes.map(|index| row.get(index))));

	rows.map(|row| {
		row.0
			.enumerate()
			.map(|(index, column)| manual.selection[index].1.solve(row).or(Ok(Value::Null)))
			.collect::<Result<Vec<Value>>>()
			.map(Row)
	})
	.collect::<Result<Vec<Row>>>()?;
}
