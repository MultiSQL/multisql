crate::util_macros::testcase!(
	(|mut glue: multisql::Glue| {
		crate::util_macros::execute!(glue, "CREATE TABLE Item (name TEXT)");
		crate::util_macros::execute!(glue, "INSERT INTO Item VALUES ('Blop mc blee'), ('B'), ('Steven the &long named$ folken!')");

		crate::util_macros::execute!(glue, "CREATE TABLE SingleItem (id INTEGER PRIMARY KEY)");
		crate::util_macros::execute!(glue, "INSERT INTO SingleItem VALUES (0)");

		crate::util_macros::execute!(glue, "CREATE TABLE NullName (name TEXT NULL)");
		crate::util_macros::execute!(glue, "INSERT INTO NullName VALUES (NULL)");

		crate::util_macros::execute!(glue, "CREATE TABLE NullNumber (number INTEGER NULL)");
		crate::util_macros::execute!(glue, "INSERT INTO NullNumber VALUES (NULL)");

		crate::util_macros::execute!(glue, "CREATE TABLE NullableName (name TEXT NULL)");
		crate::util_macros::execute!(glue, "INSERT INTO NullableName VALUES ('name')");

		crate::util_macros::assert_select!(glue, "SELECT LEFT(name, 3) AS test FROM Item" => test = Str: (String::from("Blo")),(String::from("B")),(String::from("Ste")));
		crate::util_macros::assert_select!(glue, "SELECT RIGHT(name, 10) AS test FROM Item" => test = Str: (String::from("op mc blee")), (String::from("B")), (String::from("d$ folken!")));

		// TODO: Concat crate::util_macros::assert_select!(glue, "SELECT LEFT((name + 'bobbert'), 10) AS test FROM Item" => test = Str: (String::from("Blop mc blee")), (String::from("Bbobbert")), (String::from("Steven the")));

		crate::util_macros::assert_select!(glue, "SELECT LEFT('blue', 10) AS test FROM SingleItem" => test = Str: (String::from("blue")));
		crate::util_macros::assert_select!(glue, "SELECT LEFT('blunder', 3) AS test FROM SingleItem" => test = Str: (String::from("blu")));
		crate::util_macros::assert_select!(glue, "SELECT LEFT(name, 3) AS test FROM NullName" => test = Str: (_));
		crate::util_macros::assert_select!(glue, "SELECT LEFT('Words', number) AS test FROM NullNumber" => test = Str: (_));
		crate::util_macros::assert_select!(glue, "SELECT LEFT(name, number) AS test FROM NullNumber INNER JOIN NullName ON 1 = 1" => test = Str: (_));
		crate::util_macros::assert_select!(glue, "SELECT LEFT(name, 1) AS test FROM NullableName" => test = Str: (String::from("n")));

		crate::util_macros::assert_select!(glue, "SELECT LEFT('Words', CAST(NULL AS INTEGER)) AS test FROM SingleItem" => test = Str: (_));
		crate::util_macros::assert_select!(glue, "SELECT LEFT(CAST(NULL AS TEXT), 10) AS test FROM SingleItem" => test = Str: (_));
		
		crate::util_macros::assert_error!(glue,
			"SELECT RIGHT('', 10, 10) AS test FROM SingleItem",
			multisql::ValueError::NumberOfFunctionParamsNotMatching {
				expected: 2,
				found: 3,
			}
		);
		crate::util_macros::assert_error!(glue,
			"SELECT RIGHT('') AS test FROM SingleItem",
			multisql::ValueError::NumberOfFunctionParamsNotMatching {
				expected: 2,
				found: 1,
			}
		);
		crate::util_macros::assert_error!(glue,
			"SELECT RIGHT() AS test FROM SingleItem",
			multisql::ValueError::NumberOfFunctionParamsNotMatching {
				expected: 2,
				found: 0,
			}
		);
		crate::util_macros::assert_error!(glue,
			"SELECT RIGHT(1, 1) AS test FROM SingleItem",
			multisql::ValueError::CannotConvert(multisql::Value::I64(1), "TEXT")
		);
		crate::util_macros::assert_error!(glue,
			"SELECT RIGHT('Words', 1.1) AS test FROM SingleItem",
			multisql::ValueError::CannotConvert(multisql::Value::F64(1.1), "INTEGER")
		);
		crate::util_macros::assert_error!(glue,
			"SELECT RIGHT('Words', -4) AS test FROM SingleItem",
			multisql::ValueError::BadInput(multisql::Value::I64(-4))
		);
	})
);
