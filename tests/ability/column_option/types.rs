use crate::util::*;
testcase!(test);
fn test(mut glue: multisql::Glue) {
	execute!(glue, "CREATE TABLE TableB (id BOOLEAN);");
	execute!(
		glue,
		"CREATE TABLE TableC (uid INTEGER, null_val INTEGER NULL);"
	);
	execute!(glue, "INSERT INTO TableB VALUES (FALSE);");
	execute!(glue, "INSERT INTO TableC VALUES (1, NULL);");

	assert_error!(
		glue,
		"INSERT INTO TableB SELECT uid FROM TableC;",
		multisql::ValueError::IncompatibleDataType {
			data_type: sqlparser::ast::DataType::Boolean.to_string(),
			value: format!("{:?}", multisql::Value::I64(1)),
		}
	);
	assert_error!(
		glue,
		"INSERT INTO TableC (uid) VALUES (\"A\")",
		multisql::ValueError::IncompatibleDataType {
			data_type: sqlparser::ast::DataType::Int(None).to_string(),
			value: format!("{:?}", multisql::Value::Str(String::from("A"))),
		}
	);
	assert_error!(
		glue,
		"INSERT INTO TableC VALUES (NULL, 30);",
		multisql::ValueError::NullValueOnNotNullField
	);
	assert_error!(
		glue,
		"INSERT INTO TableC SELECT null_val FROM TableC;",
		multisql::ValidateError::WrongNumberOfValues
	);
	assert_error!(
		glue,
		"UPDATE TableC SET uid = TRUE;",
		multisql::ValueError::IncompatibleDataType {
			data_type: sqlparser::ast::DataType::Int(None).to_string(),
			value: format!("{:?}", multisql::Value::Bool(true)),
		}
	);
	assert_error!(
		glue,
		"UPDATE TableC SET uid = NULL;",
		multisql::ValueError::NullValueOnNotNullField
	);
}
