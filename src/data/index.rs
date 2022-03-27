use std::cmp::Ordering;

use serde::{Deserialize, Serialize};
use sqlparser::ast::{ColumnDef, Expr, OrderByExpr};
use crate::{ExecuteError, Row, StorageInner, Value, result::Result};
use rayon::prelude::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct Index {
	pub name: String,
	pub columns: Vec<(String, bool)>,
	pub is_unique: bool,
}

impl Index {
	pub fn new(name: String,
	columns: &[OrderByExpr],
	is_unique: bool) -> Result<Self> {		
		let columns = columns.iter().map(|OrderByExpr{expr, asc, ..}| { // TODO: Check that these are correct
			if let Expr::Identifier(ident) = expr {
				let asc = asc.unwrap_or(true);
				let ident = ident.value;
				Ok((ident, asc))
			} else {
				Err(ExecuteError::QueryNotSupported.into()) // TODO: Be more specific
			}
		}).collect::<Result<Vec<(String, bool)>>>()?;
		Ok(Self{
			name,
			columns,
			is_unique
		})
	}
	pub async fn reset(&self, storage: &mut StorageInner, table: &str, column_defs: &[ColumnDef]) -> Result<()> {
		let rows = storage.scan_data(table).await?.collect::<Result<Vec<(Value, Row)>>>()?;
		let column_defs = column_defs.iter().enumerate();
		let column_indexes: Vec<(usize, bool)> = self.columns.iter().map(|(column, asc)| (column_defs.find(|(index, def)| &def.name.value == column).unwrap().0, asc)).collect(); // TODO: Handle
		let column_indexes = column_indexes.into_iter();

		let mut rows = rows.into_iter().map(|row| row.0).collect();
		rows.par_sort_unstable_by(|(a_key, a_values), (b_key, b_values)| {
			column_indexes.find_map(|(index, asc)| {
				let order = a_values[index].cmp(b_values[index]);
				if order.is_eq {
					None
				} else if asc {
					order.reverse()
				} else {
					order
				}
			}).unwrap_or(Ordering::Equal)
		});
		let keys = rows.into_iter().map(|(key, values)| key);

		storage.update_index(&table, &self.name, keys);
		Ok(())
	}
}
