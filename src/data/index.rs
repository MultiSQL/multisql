use {
	crate::{result::Result, Column, Ingredient, Method, Recipe, StorageInner, Value},
	rayon::prelude::*,
	serde::{Deserialize, Serialize},
	std::{cmp::Ordering, collections::HashMap},
};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Index {
	pub name: String,
	pub column: String,
	pub is_unique: bool,
}

#[derive(Clone, Debug)]
pub enum IndexFilter {
	LessThan(String, Value), // Index, Min, Max
	MoreThan(String, Value), // Index, Min, Max
	Inner(Box<IndexFilter>, Box<IndexFilter>),
	Outer(Box<IndexFilter>, Box<IndexFilter>),
}

impl Index {
	pub fn new(name: String, column: String, is_unique: bool) -> Self {
		Self {
			name,
			column,
			is_unique,
		}
	}
	pub async fn reset(
		&self,
		storage: &mut StorageInner,
		table: &str,
		columns: &[Column],
	) -> Result<()> {
		let rows = storage
			.scan_data(table)
			.await?;
		let column_index: usize = columns
			.iter()
			.enumerate()
			.find_map(|(index, def)| (def.name == self.column).then(|| index))
			.unwrap(); // TODO: Handle

		let mut rows: Vec<(Value, Vec<Value>)> =
			rows.into_iter().map(|(key, row)| (key, row.0)).collect();
		rows.par_sort_unstable_by(|(_, a_values), (_, b_values)| {
			a_values[column_index]
				.partial_cmp(&b_values[column_index])
				.unwrap_or(Ordering::Equal)
		});
		let keys = rows
			.into_iter()
			.map(|(key, mut values)| (values.swap_remove(column_index), key))
			.collect();

		storage.update_index(table, &self.name, keys).await
	}
}

impl Recipe {
	pub fn reduce_by_index_filter(
		self,
		indexed_columns: HashMap<usize, (String, String)>,
	) -> (Self, Option<HashMap<String, IndexFilter>>) {
		// TODO: OR & others
		use IndexFilter::*;
		match self {
			Recipe::Ingredient(_) => (),
			Recipe::Method(ref method) => match *method.clone() {
				Method::BinaryOperation(operator, left, right)
					if operator as usize == Value::and as usize =>
				{
					let (left, left_filters) = left.reduce_by_index_filter(indexed_columns.clone());
					let (right, right_filters) = right.reduce_by_index_filter(indexed_columns);
					return (
						Recipe::Method(Box::new(Method::BinaryOperation(operator, left, right))),
						match (left_filters, right_filters) {
							(Some(filters), None) | (None, Some(filters)) => Some(filters),
							(Some(left_filters), Some(mut right_filters)) => Some(
								left_filters
									.into_iter()
									.map(|(table, filter)| {
										(
											table.clone(),
											match right_filters.remove(&table) {
												Some(right) => {
													Inner(Box::new(filter), Box::new(right))
												}
												None => filter,
											},
										)
									})
									.collect::<HashMap<String, IndexFilter>>()
									.into_iter()
									.chain(right_filters.into_iter())
									.collect::<HashMap<String, IndexFilter>>(),
							),
							(None, None) => None, // TODO: Don't unnecessarily rebuild
						},
					);
				}
				Method::BinaryOperation(
					operator,
					Recipe::Ingredient(Ingredient::Column(column)),
					Recipe::Ingredient(Ingredient::Value(value)),
				) if operator as usize == Value::eq as usize => {
					{
						if let Some((table, index)) = indexed_columns.get(&column) {
							let mut filters = HashMap::new();
							filters.insert(
								table.clone(),
								Inner(
									Box::new(LessThan(index.clone(), value.inc())),
									Box::new(MoreThan(index.clone(), value)),
								),
							); // Eh; TODO: Improve
							return (Recipe::TRUE, Some(filters));
						}
					}
				}
				Method::BinaryOperation(
					operator,
					Recipe::Ingredient(Ingredient::Column(column)),
					Recipe::Ingredient(Ingredient::Value(value)),
				) if operator as usize == Value::gt_eq as usize => {
					if let Some((table, index)) = indexed_columns.get(&column) {
						let mut filters = HashMap::new();
						filters.insert(table.clone(), MoreThan(index.clone(), value));
						return (Recipe::TRUE, Some(filters));
					}
				}
				Method::BinaryOperation(
					operator,
					Recipe::Ingredient(Ingredient::Column(column)),
					Recipe::Ingredient(Ingredient::Value(value)),
				) if operator as usize == Value::gt as usize => {
					if let Some((table, index)) = indexed_columns.get(&column) {
						let mut filters = HashMap::new();
						filters.insert(table.clone(), MoreThan(index.clone(), value.inc()));
						return (Recipe::TRUE, Some(filters));
					}
				}
				Method::BinaryOperation(
					operator,
					Recipe::Ingredient(Ingredient::Column(column)),
					Recipe::Ingredient(Ingredient::Value(value)),
				) if operator as usize == Value::lt as usize => {
					if let Some((table, index)) = indexed_columns.get(&column) {
						let mut filters = HashMap::new();
						filters.insert(table.clone(), LessThan(index.clone(), value));
						return (Recipe::TRUE, Some(filters));
					}
				}
				Method::BinaryOperation(
					operator,
					Recipe::Ingredient(Ingredient::Column(column)),
					Recipe::Ingredient(Ingredient::Value(value)),
				) if operator as usize == Value::lt_eq as usize => {
					if let Some((table, index)) = indexed_columns.get(&column) {
						let mut filters = HashMap::new();
						filters.insert(table.clone(), LessThan(index.clone(), value.inc()));
						return (Recipe::TRUE, Some(filters));
					}
				}
				_ => (),
			},
		}
		(self, None)
	}
}
