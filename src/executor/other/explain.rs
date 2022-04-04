use crate::{ExecuteError, Payload, Row, Schema};
use crate::{Result, StorageInner};
use sqlparser::ast::ObjectName;

pub(crate) async fn explain(
	storages: &[(String, &mut StorageInner)],
	object: &ObjectName,
) -> Result<Payload> {
	println!("{:?}", object);

	let mut name_vec = object.0.clone();
	let (store_name, opt_table_name) = match name_vec.len() {
		2 => (name_vec.remove(0).value, Some(name_vec.remove(0).value)),
		1 => {
			let name = name_vec.remove(0).value;
			if storages.iter().any(|(store, _)| store == &name) {
				(name, None)
			} else {
				(storages[0].0.clone(), Some(name))
			}
		}
		_ => return Err(ExecuteError::ObjectNotRecognised.into()),
	};

	let store = storages
		.iter()
		.find_map(|(name, store)| (name == &store_name).then(|| store))
		.ok_or_else(|| ExecuteError::ObjectNotRecognised)?;
	if let Some(table_name) = opt_table_name {
		let Schema { column_defs, .. } = store
			.fetch_schema(&table_name)
			.await?
			.ok_or_else(|| ExecuteError::ObjectNotRecognised)?;
		let columns = column_defs
			.iter()
			.map(|column_def| {
				(
					column_def.name.value.clone().into(),
					column_def.data_type.to_string().into(),
				)
			})
			.map(|(name, data_type)| Row(vec![name, data_type]))
			.collect();
		Ok(Payload::Select {
			labels: vec![String::from("column"), String::from("data_type")],
			rows: columns,
		})
	} else {
		let tables = store
			.scan_schemas()
			.await?
			.into_iter()
			.map(|Schema { table_name, .. }| Row(vec![table_name.into()]))
			.collect();
		Ok(Payload::Select {
			labels: vec![String::from("table")],
			rows: tables,
		})
	}
}
