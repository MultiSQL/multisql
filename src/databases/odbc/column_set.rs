use {
	super::base::convert_table_name,
	crate::{Cast, Column, DBBase, DBMut, Error, ODBCDatabase, Result, Row, Value, ValueType},
	async_trait::async_trait,
	odbc_api::{
		buffers::{
			AnyColumnBuffer, BufferDescription, BufferKind, ColumnarBuffer, NullableSliceMut,
			TextColumn,
		},
		parameter::InputParameter,
		Bit, IntoParameter,
	},
};

impl Into<BufferKind> for ValueType {
	fn into(self) -> BufferKind {
		match self {
			ValueType::Str => BufferKind::Text {
				max_str_len: 255, // Arbitrary!!! Could be a big problem..
			},
			ValueType::I64 => BufferKind::I64,
			ValueType::F64 => BufferKind::F64,
			ValueType::Bool => BufferKind::Bit,
			ValueType::Timestamp => BufferKind::Timestamp,
			ValueType::U64 => BufferKind::I64, // Unsafe
			_ => unimplemented!(),
		}
	}
}

struct ColumnValues {
	index: usize,
	value_type: ValueType,
	values: Vec<Value>,
	size: usize,
}
impl TryInto<AnyColumnBuffer> for ColumnValues {
	type Error = Error;
	fn try_into(self) -> Result<AnyColumnBuffer> {
		let description = BufferDescription {
			nullable: true,
			kind: self.value_type.into(),
		};
		let mut buffer = AnyColumnBuffer::from_description(self.size, description);
		match &mut buffer {
			AnyColumnBuffer::NullableI64(column) => {
				let values = self
					.values
					.into_iter()
					.map(|cell| {
						Ok(if matches!(cell, Value::Null) {
							None
						} else {
							Some(cell.cast()?)
						})
					})
					.collect::<Result<Vec<Option<i64>>>>()?;
				column.writer_n(self.size).write(values.into_iter());
			}
			AnyColumnBuffer::Text(column) => {
				let values = self
					.values
					.into_iter()
					.map(|cell| {
						Ok(if matches!(cell, Value::Null) {
							None
						} else {
							Some(cell.cast()?)
						})
					})
					.collect::<Result<Vec<Option<String>>>>()?;
				column.writer_n(self.size).write(
					values
						.iter()
						.map(|text| text.as_ref().map(|text| text.as_bytes())),
				);
			}
			_ => unimplemented!(),
		}
		Ok(buffer)
	}
}
pub(crate) struct ColumnSet {
	columns: Vec<ColumnValues>,
}

impl ColumnSet {
	pub fn new(rows: Vec<Vec<Value>>, size: usize) -> Self {
		let columns: Vec<(Vec<Value>, Option<ValueType>)> =
			rows[0].iter().map(|_| (Vec::new(), None)).collect();
		let columns = rows.into_iter().fold(columns, |mut columns, row| {
			row.into_iter().enumerate().for_each(|(index, cell)| {
				if columns[index].1.is_none() && !matches!(cell, Value::Null) {
					columns[index].1 = Some((&cell).into());
				}
				columns[index].0.push(cell);
			});
			columns
		});
		let columns = columns
			.into_iter()
			.enumerate()
			.filter_map(|(index, (values, value_type))| {
				value_type.map(|value_type| ColumnValues {
					index,
					value_type,
					values,
					size,
				})
			})
			.collect();
		Self { columns }
	}
	pub fn query(&self, table: &str, columns: &[&str]) -> String {
		let columns: Vec<&str> = columns
			.into_iter()
			.enumerate()
			.filter(|(index, _)| self.columns.iter().any(|col| index == &col.index))
			.map(|(_, column)| *column)
			.collect();
		let placeholders: Vec<&str> = columns.iter().map(|_| "?").collect();
		format!(
			"INSERT INTO {table} ({columns}) VALUES ({placeholders})",
			table = table,
			columns = columns.join(", "),
			placeholders = placeholders.join(", ")
		)
	}
}
impl TryInto<ColumnarBuffer<AnyColumnBuffer>> for ColumnSet {
	type Error = Error;
	fn try_into(self) -> Result<ColumnarBuffer<AnyColumnBuffer>> {
		let buffers = self
			.columns
			.into_iter()
			.enumerate()
			.map(|(index, column)| column.try_into().map(|buffer| (index as u16, buffer)))
			.collect::<Result<Vec<_>>>()?;
		Ok(ColumnarBuffer::new(buffers))
	}
}
