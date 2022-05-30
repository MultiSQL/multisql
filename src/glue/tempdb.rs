use {crate::Value, std::collections::HashMap};

#[derive(Clone)]
pub struct TempDB {
	variables: HashMap<String, Value>,
	tables: HashMap<String, (Vec<String>, Vec<Vec<Value>>)>,
}

impl Default for TempDB {
	fn default() -> Self {
		TempDB {
			variables: HashMap::new(),
			tables: HashMap::new(),
		}
	}
}

impl TempDB {
	pub fn get_variable(&self, name: &str) -> Option<&Value> {
		self.variables.get(name)
	}
	pub fn set_variable(&mut self, name: String, value: Value) -> Option<Value> {
		self.variables.insert(name, value)
	}
	pub fn get_table(&self, name: &str) -> Option<&(Vec<String>, Vec<Vec<Value>>)> {
		self.tables.get(name)
	}
	pub fn set_table(
		&mut self,
		name: String,
		data: (Vec<String>, Vec<Vec<Value>>),
	) -> Option<(Vec<String>, Vec<Vec<Value>>)> {
		self.tables.insert(name, data)
	}
}
