crate::util_macros::testcase!(
	(|mut glue: multisql::Glue| {
		crate::util_macros::execute!(glue, "CREATE TABLE TableB (id BOOLEAN);");
		crate::util_macros::execute!(glue, "CREATE TABLE TableC (uid INTEGER, null_val INTEGER NULL);");
		crate::util_macros::execute!(glue, "INSERT INTO TableB VALUES (FALSE);");
		crate::util_macros::execute!(glue, "INSERT INTO TableC VALUES (1, NULL);");

		crate::util_macros::assert_error!(glue,
			"INSERT INTO TableB SELECT uid FROM TableC;",
			multisql::ValueError::IncompatibleDataType {
				data_type: sqlparser::ast::DataType::Boolean.to_string(),
				value: format!("{:?}", multisql::Value::I64(1)),
			}
		);
		crate::util_macros::assert_error!(glue,
			"INSERT INTO TableC (uid) VALUES (\"A\")",
			multisql::ValueError::IncompatibleDataType {
				data_type: sqlparser::ast::DataType::Int(None).to_string(),
				value: format!("{:?}", multisql::Value::Str(String::from("A"))),
			}
		);
		crate::util_macros::assert_error!(glue,
			"INSERT INTO TableC VALUES (NULL, 30);",
			multisql::ValueError::NullValueOnNotNullField
		);
		crate::util_macros::assert_error!(glue,
			"INSERT INTO TableC SELECT null_val FROM TableC;",
			multisql::ValidateError::WrongNumberOfValues
		);
		crate::util_macros::assert_error!(glue,
			"UPDATE TableC SET uid = TRUE;",
			multisql::ValueError::IncompatibleDataType {
				data_type: sqlparser::ast::DataType::Int(None).to_string(),
				value: format!("{:?}", multisql::Value::Bool(true)),
			}
		);
		crate::util_macros::assert_error!(glue,
			"UPDATE TableC SET uid = NULL;",
			multisql::ValueError::NullValueOnNotNullField
		);
	})
);
