use {
	crate::{Column, NullOrd, Result, Row, StorageInner, ValidateError, Value},
	std::cmp::Ordering,
};

macro_rules! some_or_continue {
	($option: expr) => {
		match $option {
			Some(value) => value,
			None => return Some(Ok(())),
		}
	};
}
macro_rules! some_or {
	($option: expr, $or: block) => {
		match $option {
			Some(value) => value,
			None => $or,
		}
	};
}

pub async fn validate_unique(
	storage: &StorageInner,
	table_name: &str,
	column_defs: &[Column],
	rows: &[Row],
	ignore_keys: Option<&[Value]>,
) -> Result<()> {
	let unique_columns: Vec<usize> = column_defs
		.iter()
		.enumerate()
		.filter_map(|(index, column_def)| {
			if column_def.is_unique {
				Some(index)
			} else {
				None
			}
		})
		.collect();
	let mut existing_values: Vec<Vec<Value>> = vec![vec![]; unique_columns.len()];

	storage
		.scan_data(table_name)
		.await?
		.try_for_each::<_, Result<_>>(|result| {
			let (key, row) = result?;
			if let Some(ignore_keys) = ignore_keys {
				if ignore_keys.iter().any(|ignore_key| ignore_key == &key) {
					return Ok(());
				}
			}
			let row = row.0;
			unique_columns
				.iter()
				.enumerate()
				.map(|(index, row_index)| {
					existing_values
						.get_mut(index)?
						.push(row.get(*row_index)?.clone());
					Some(())
				})
				.collect::<Option<()>>()
				.ok_or_else(|| ValidateError::UnreachableUniqueValues.into())
		})?;

	let mut new_values: Vec<Vec<Value>> = vec![vec![]; unique_columns.len()];
	rows.iter().try_for_each::<_, Result<_>>(|row| {
		unique_columns
			.iter()
			.enumerate()
			.map(|(index, row_index)| {
				new_values
					.get_mut(index)?
					.push(row.0.get(*row_index)?.clone());
				Some(())
			})
			.collect::<Option<()>>()
			.ok_or_else(|| ValidateError::UnreachableUniqueValues.into())
	})?;
	let mut existing_values_iter = existing_values.into_iter();
	new_values
		.into_iter()
		.map(|mut new_values| {
			let mut existing_values = existing_values_iter.next()?;

			existing_values.sort_unstable_by(|value_l, value_r| {
				value_l.partial_cmp(value_r).unwrap_or(Ordering::Equal)
			});
			new_values.sort_unstable_by(|value_l, value_r| {
				value_l.partial_cmp(value_r).unwrap_or(Ordering::Equal)
			});

			let mut existing_values = existing_values.into_iter();
			let mut new_values = new_values.into_iter();

			let mut new_value = some_or_continue!(new_values.next());
			let mut existing_value = some_or!(existing_values.next(), {
				loop {
					let new_new = some_or_continue!(new_values.next());
					if new_new == new_value {
						return Some(Err(ValidateError::DuplicateEntryOnUniqueField.into()));
					}
					new_value = new_new;
				}
			});

			loop {
				match existing_value.null_cmp(&new_value) {
					Some(Ordering::Equal) => {
						return Some(Err(ValidateError::DuplicateEntryOnUniqueField.into()))
					}
					Some(Ordering::Greater) => {
						let new_new = some_or_continue!(new_values.next());
						if new_new == new_value {
							return Some(Err(ValidateError::DuplicateEntryOnUniqueField.into()));
						}
						new_value = new_new;
					}
					Some(Ordering::Less) => {
						existing_value = some_or!(existing_values.next(), {
							loop {
								let new_new = some_or_continue!(new_values.next());
								if new_new == new_value {
									return Some(Err(
										ValidateError::DuplicateEntryOnUniqueField.into()
									));
								}
								new_value = new_new;
							}
						});
					}
					None => {
						let new_new = some_or_continue!(new_values.next());
						if new_new == new_value {
							return Some(Err(ValidateError::DuplicateEntryOnUniqueField.into()));
						}
						new_value = new_new;
						existing_value = some_or!(existing_values.next(), {
							loop {
								let new_new = some_or_continue!(new_values.next());
								if new_new == new_value {
									return Some(Err(
										ValidateError::DuplicateEntryOnUniqueField.into()
									));
								}
								new_value = new_new;
							}
						});
					}
				}
			}
		})
		.collect::<Option<Result<()>>>()
		.ok_or(ValidateError::UnreachableUniqueValues)?
}
