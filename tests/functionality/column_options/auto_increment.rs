crate::util_macros::testcase!(
	(|mut glue: multisql::Glue| {
		crate::util_macros::execute!(glue, "CREATE TABLE Test (id INTEGER AUTO_INCREMENT NOT NULL, name TEXT)");
		crate::util_macros::execute!(glue, "INSERT INTO Test (name) VALUES ('test1')");
		
		crate::util_macros::assert_select!(glue, r#"
			SELECT
				*
			FROM
				Test
			"# => id = I64, name = Str:
				(1, String::from("test1"))
		);

		crate::util_macros::execute!(glue, "INSERT INTO Test (name) VALUES ('test2'), ('test3')");
		
		crate::util_macros::assert_select!(glue, r#"
			SELECT
				*
			FROM
				Test
			"# => id = I64, name = Str:
				(1, String::from("test1")),
				(2, String::from("test2")),
				(3, String::from("test3"))
		);

		crate::util_macros::execute!(glue, "INSERT INTO Test (name, id) VALUES ('test4', NULL)");
		
		crate::util_macros::assert_select!(glue, r#"
			SELECT
				*
			FROM
				Test
			"# => id = I64, name = Str:
				(1, String::from("test1")),
				(2, String::from("test2")),
				(3, String::from("test3")),
				(4, String::from("test4"))
		);

		crate::util_macros::execute!(glue, "INSERT INTO Test (name, id) VALUES ('test5', 6)");
		
		crate::util_macros::assert_select!(glue, r#"
			SELECT
				*
			FROM
				Test
			"# => id = I64, name = Str:
				(1, String::from("test1")),
				(2, String::from("test2")),
				(3, String::from("test3")),
				(4, String::from("test4")),
				(6, String::from("test5"))
		);

		crate::util_macros::execute!(glue, "INSERT INTO Test (name) VALUES ('test6')");
		
		crate::util_macros::assert_select!(glue, r#"
			SELECT
				*
			FROM
				Test
			"# => id = I64, name = Str:
				(1, String::from("test1")),
				(2, String::from("test2")),
				(3, String::from("test3")),
				(4, String::from("test4")),
				(6, String::from("test5")),
				(5, String::from("test6"))
		);

		crate::util_macros::execute!(glue, "INSERT INTO Test (name) VALUES ('test7')");
		
		crate::util_macros::assert_select!(glue, r#"
			SELECT
				*
			FROM
				Test
			"# => id = I64, name = Str:
				(1, String::from("test1")),
				(2, String::from("test2")),
				(3, String::from("test3")),
				(4, String::from("test4")),
				(6, String::from("test5")),
				(5, String::from("test6")),
				(6, String::from("test7"))
		);
		crate::util_macros::execute!(glue, "CREATE TABLE TestUnique (id INTEGER AUTO_INCREMENT NOT NULL UNIQUE, name TEXT)");
		crate::util_macros::execute!(glue, "INSERT INTO TestUnique (name, id) VALUES ('test1', NULL), ('test2', 3)");

		crate::util_macros::assert_select!(glue, r#"
			SELECT
				*
			FROM
				TestUnique
			"# => id = I64, name = Str:
				(1, String::from("test1")),
				(3, String::from("test2"))
		);

		{
			let _result: Result<multisql::Payload, multisql::Error> = Err(multisql::ValidateError::DuplicateEntryOnUniqueField.into());
			assert!(matches!(
				glue.execute("INSERT INTO TestUnique (name) VALUES ('test3'), ('test4')"),
				_result
			));
		}

		crate::util_macros::assert_select!(glue, r#"
			SELECT
				*
			FROM
				TestUnique
			"# => id = I64, name = Str:
				(1, String::from("test1")),
				(3, String::from("test2"))
		);

		crate::util_macros::execute!(glue, "CREATE TABLE TestUniqueSecond (id INTEGER AUTO_INCREMENT NOT NULL UNIQUE, name TEXT)");
		{
			let _result: Result<multisql::Payload, multisql::Error> = Err(multisql::ValidateError::DuplicateEntryOnUniqueField.into());
			assert!(matches!(
				glue.execute("INSERT INTO TestUniqueSecond (name, id) VALUES ('test1', NULL), ('test2', 3), ('test3', NULL), ('test4', NULL)"),
				_result
			));
		}
		crate::util_macros::execute!(glue, "CREATE TABLE TestInsertSelect (id INTEGER AUTO_INCREMENT NOT NULL, name TEXT)");
		crate::util_macros::execute!(glue, r#"INSERT INTO TestInsertSelect (name) SELECT name FROM Test"#);
		{
			let _result: Result<multisql::Payload, multisql::Error> = Err(multisql::AlterError::UnsupportedDataTypeForAutoIncrementColumn(String::from("id"), String::from("TEXT")).into());
			assert!(matches!(
				glue.execute("CREATE TABLE TestText (id TEXT AUTO_INCREMENT NOT NULL UNIQUE, name TEXT)"),
				_result
			));
		}

		/*crate::util_macros::assert_select!(glue, r#"
			SELECT
				*
			FROM
				TestInsertSelect
			"# => id = I64, name = Str:
				(1, String::from("test1")),
				(2, String::from("test2")),
				(3, String::from("test3")),
				(4, String::from("test4")),
				(5, String::from("test5")),
				(6, String::from("test6")),
				(7, String::from("test7"))
		); Tempremental
		*/

	})
);
