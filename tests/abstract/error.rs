crate::util_macros::testcase!(
	(|mut glue: multisql::Glue| {
	crate::util_macros::execute!(glue, "CREATE TABLE TableA (id INTEGER);");
	crate::util_macros::execute!(glue, "INSERT INTO TableA (id) VALUES (1);");

		crate::util_macros::assert_error!(glue,
			"COMMIT;",
			multisql::ExecuteError::QueryNotSupported,
		);
		crate::util_macros::assert_error!(glue,
			"INSERT INTO Nothing VALUES (1);",
			multisql::ExecuteError::TableNotExists,
		);
		crate::util_macros::assert_error!(glue,
			"UPDATE Nothing SET a = 1;",
			multisql::ExecuteError::TableNotExists,
		);
		crate::util_macros::assert_error!(glue,
			"SELECT * FROM Nothing;",
			multisql::FetchError::TableNotFound(String::from("Nothing")),
		);
		crate::util_macros::assert_error!(glue,
			"SELECT * FROM TableA JOIN (SELECT * FROM TableB) as TableC ON 1 = 1",
			multisql::JoinError::UnimplementedTableType,
		);
		crate::util_macros::assert_error!(glue,
			"SELECT * FROM TableA JOIN TableA USING (id);",
			multisql::JoinError::UsingOnJoinNotSupported,
		);
		crate::util_macros::assert_error!(glue,
			"SELECT * FROM TableA WHERE id = (SELECT id FROM TableA WHERE id = 2);",
			multisql::ManualError::UnimplementedSubquery,
		);
		crate::util_macros::assert_error!(glue,
			"SELECT * FROM TableA WHERE noname = 1;",
			multisql::RecipeError::MissingColumn(vec![String::from("noname")]),
		);
		crate::util_macros::assert_error!(glue,
			"INSERT INTO TableA (id2) VALUES (1);",
			multisql::ValidateError::ColumnNotFound(String::from("id2")),
		);
		crate::util_macros::assert_error!(glue,
			"INSERT INTO TableA (id2, id) VALUES (100);",
			multisql::ValidateError::ColumnNotFound(String::from("id2")),
		);
		crate::util_macros::assert_error!(glue,
			"INSERT INTO TableA VALUES (100), (100, 200);",
			multisql::ValidateError::WrongNumberOfValues,
		);
		crate::util_macros::assert_error!(glue,
			"SELECT * FROM TableA Where id = X'123';",
			multisql::ValueError::UnimplementedLiteralType,
		);
	})
);
