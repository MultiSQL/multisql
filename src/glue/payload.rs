use crate::{Payload, Row};

impl Payload {
	pub fn unwrap_rows(self) -> Vec<Row> {
		if let Payload::Select { rows, .. } = self {
			rows
		} else {
			panic!("Expected Select!")
		}
	}
}
