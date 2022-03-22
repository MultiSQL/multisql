use {
	crate::{
		executor::types::{ComplexColumnName, Row},
		MetaRecipe, PlannedRecipe, RecipeUtilities, Result, Value,
	},
	sqlparser::ast::OrderByExpr,
	std::cmp::Ordering,
	rayon::prelude::*
};

pub struct Order(Vec<PlannedOrderItem>);
impl Order {
	pub fn new(order_by: Vec<OrderByExpr>, columns: &Vec<ComplexColumnName>) -> Result<Self> {
		let order_items = order_by
			.into_iter()
			.map(|order_by_item| PlannedOrderItem::new(order_by_item, columns))
			.collect::<Result<Vec<PlannedOrderItem>>>()?;
		Ok(Order(order_items))
	}
	pub fn execute(self, rows: Vec<Row>) -> Result<Vec<Row>> { // TODO: Optimise
		if self.0.is_empty() {
			return Ok(rows);
		}
		
		let (order_terms, order_item_recipes): (Vec<OrderTerm>, Vec<PlannedRecipe>) = self
			.0
			.into_iter()
			.map(|planned_order_item| {
				let PlannedOrderItem(order_term, recipe) = planned_order_item;
				(order_term, recipe)
			})
			.unzip();
		let order_terms = OrderTerms(order_terms);

		let mut order_rows = rows
			.into_par_iter()
			.map(|row| {
				let order_row = order_item_recipes
					.clone()
					.into_iter()
					.map(|recipe| recipe.simplify_by_row(&row)?.confirm())
					.collect::<Result<Vec<Value>>>();
				order_row.map(|order_row| (row, order_row))
			})
			.collect::<Result<Vec<(Row, Vec<Value>)>>>()?;

		order_rows.par_sort_unstable_by(|(_, order_row_a), (_, order_row_b)| {
			order_terms.sort(order_row_a, order_row_b)
		});
		Ok(order_rows.into_iter().map(|(row, _)| row).collect())
	}
}

struct PlannedOrderItem(OrderTerm, PlannedRecipe);
impl PlannedOrderItem {
	pub fn new(order_by_item: OrderByExpr, columns: &Vec<ComplexColumnName>) -> Result<Self> {
		let OrderByExpr {
			expr,
			asc,
			nulls_first,
		} = order_by_item;
		let recipe = PlannedRecipe::new(MetaRecipe::new(expr)?.simplify_by_basic()?, columns)?;
		let is_asc = asc.unwrap_or(true);
		let prefer_nulls = nulls_first.unwrap_or(false);

		Ok(PlannedOrderItem(
			OrderTerm {
				is_asc,
				prefer_nulls,
			},
			recipe,
		))
	}
}

#[derive(Clone)]
struct OrderTerm {
	pub is_asc: bool,
	pub prefer_nulls: bool,
}
impl OrderTerm {
	pub fn sort(&self, order_item_a: &Value, order_item_b: &Value) -> Option<Ordering> {
		let order = match (order_item_a, order_item_b) {
			(Value::Null, Value::Null) => Ordering::Equal,
			(Value::Null, _) | (_, Value::Null) => {
				if self.prefer_nulls {
					Ordering::Greater
				} else {
					Ordering::Less
				}
			}
			(other_a, other_b) => other_a.partial_cmp(other_b).unwrap_or(Ordering::Equal),
		};
		if order == Ordering::Equal {
			None
		} else if self.is_asc {
			Some(order)
		} else {
			Some(order.reverse())
		}
	}
}

struct OrderTerms(Vec<OrderTerm>);

impl OrderTerms {
	pub fn sort(&self, order_items_a: &Vec<Value>, order_items_b: &Vec<Value>) -> Ordering {
		order_items_a
			.iter()
			.zip(order_items_b)
			.zip(self.0.clone())
			.find_map(|((order_item_a, order_item_b), order_term)| {
				order_term.sort(order_item_a, order_item_b)
			})
			.unwrap_or(Ordering::Equal)
	}
}
