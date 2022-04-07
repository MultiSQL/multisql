use crate::{ExecuteError, Payload, Row, Schema, Value};
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
			if name == "ALL" {
				let databases = storages
					.iter()
					.map(|(name, _)| Row(vec![name.clone().into()]))
					.collect();
				return Ok(Payload::Select {
					labels: vec![String::from("database")],
					rows: databases,
				});
			}
			if name == "ALL_TABLE" {
				let mut tables = vec![];
				for (name, store) in storages.iter() {
					tables.extend(
						get_tables(store)
							.await?
							.into_iter()
							.map(|table| Row(vec![name.clone().into(), table])),
					);
				}
				return Ok(Payload::Select {
					labels: vec![String::from("database"), String::from("table")],
					rows: tables,
				});
			} else if storages.iter().any(|(store, _)| store == &name) {
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
			.map(|column| {
				(
					column.name.clone().into(),
					column.data_type.to_string().into(),
				)
			})
			.map(|(name, data_type)| Row(vec![name, data_type]))
			.collect();
		Ok(Payload::Select {
			labels: vec![String::from("column"), String::from("data_type")],
			rows: columns,
		})
	} else {
		Ok(Payload::Select {
			labels: vec![String::from("table")],
			rows: get_tables(&store)
				.await?
				.into_iter()
				.map(|table| Row(vec![table]))
				.collect(),
		})
	}
}

async fn get_tables(store: &&mut StorageInner) -> Result<Vec<Value>> {
	Ok(store
		.scan_schemas()
		.await?
		.into_iter()
		.map(|Schema { table_name, .. }| table_name.into())
		.collect())
}
