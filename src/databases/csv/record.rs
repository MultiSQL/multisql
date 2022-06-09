use {crate::Value, csv::StringRecord};

pub fn string_record_to_row(record: &StringRecord) -> Vec<Value> {
	record.iter().map(csv_cell_to_value).collect()
}

fn csv_cell_to_value(cell: &str) -> Value {
	let cell = cell.to_string();
	cell.parse::<bool>()
		.map(|v| Value::Bool(v))
		.or_else(|_| cell.parse::<u64>().map(|v| Value::U64(v)))
		.or_else(|_| cell.parse::<i64>().map(|v| Value::I64(v)))
		.or_else(|_| cell.parse::<f64>().map(|v| Value::F64(v)))
		.unwrap_or_else(|_| Value::Str(cell))
}
