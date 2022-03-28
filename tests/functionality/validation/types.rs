crate::util_macros::testcase!(
	(|mut glue: multisql::Glue| {
		crate::util_macros::execute!(glue, "CREATE TABLE TableB (id BOOLEAN);");
		crate::util_macros::execute!(glue, "CREATE TABLE TableC (uid INTEGER, null_val INTEGER NULL);");
		crate::util_macros::execute!(glue, "INSERT INTO TableB VALUES (FALSE);");
		crate::util_macros::execute!(glue, "INSERT INTO TableC VALUES (1, NULL);");

		let test_cases: Vec<(_, multisql::Error)> = vec![
			(
				"INSERT INTO TableB SELECT uid FROM TableC;",
				multisql::ValueError::IncompatibleDataType {
					data_type: multisql::parser::ast::DataType::Boolean.to_string(),
					value: format!("{:?}", multisql::Value::I64(1)),
				}.into()
			),
			(
				"INSERT INTO TableC (uid) VALUES (\"A\")",
				multisql::ValueError::IncompatibleDataType {
					data_type: multisql::parser::ast::DataType::Int(None).to_string(),
					value: format!("{:?}", multisql::Value::Str(String::from("A"))),
				}.into()
			),
			(
				"INSERT INTO TableC VALUES (NULL, 30);",
				multisql::ValueError::NullValueOnNotNullField.into()
			),
			(
				"INSERT INTO TableC SELECT null_val FROM TableC;",
				multisql::ValidateError::WrongNumberOfValues.into()
			),
			(
				"UPDATE TableC SET uid = TRUE;",
				multisql::ValueError::IncompatibleDataType {
					data_type: multisql::parser::ast::DataType::Int(None).to_string(),
					value: format!("{:?}", multisql::Value::Bool(true)),
				}.into()
			),
			(
				"UPDATE TableC SET uid = NULL;",
				multisql::ValueError::NullValueOnNotNullField.into()
			)
		];

		for (sql, _expected) in test_cases.into_iter() {
			assert!(matches!(glue.execute(sql), Err(_expected)));
		}
	})
);
