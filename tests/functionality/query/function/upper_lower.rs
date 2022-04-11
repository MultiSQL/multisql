crate::util_macros::testcase!(
	(|mut glue: multisql::Glue| {

		crate::util_macros::assert_success!(glue,
			"CREATE TABLE Item (name TEXT, opt_name TEXT NULL)",
			multisql::Payload::Create
		);
		crate::util_macros::assert_success!(glue,
			"INSERT INTO Item VALUES ('abcd', 'efgi'), ('Abcd', NULL), ('ABCD', 'EfGi')",
			multisql::Payload::Insert(3)
		);
		crate::util_macros::assert_select!(glue,
			"SELECT name FROM Item WHERE LOWER(name) = 'abcd'" => name = Str:
			(String::from("abcd")),
			(String::from("Abcd")),
			(String::from("ABCD"))
		);
		crate::util_macros::assert_select!(glue,
			"SELECT LOWER(name) AS lower, UPPER(name) as upper FROM Item;" => lower = Str, upper = Str:
			((String::from("abcd")), (String::from("ABCD"))),
			((String::from("abcd")), (String::from("ABCD"))),
			((String::from("abcd")), (String::from("ABCD")))
		);
		crate::util_macros::assert_select!(glue,
            "VALUES (LOWER('Abcd'), UPPER('abCd'))" => unnamed_0 = Str, unnamed_1 = Str:
			((String::from("abcd")), (String::from("ABCD")))
		);
		/* TODO: Null test
		crate::util_macros::assert_select!(glue,
			"SELECT LOWER(opt_name) AS lower, UPPER(opt_name) as upper FROM Item;" => lower = Str, upper = Str:
			((String::from("efgi")), (String::from("EFGI"))),
			(), (),
			((String::from("efgi")), (String::from("EFGI")))
		);*/
		crate::util_macros::assert_error!(glue,
			"SELECT LOWER() FROM Item",
			multisql::ValueError::NumberOfFunctionParamsNotMatching {
				expected: 1,
				found: 0,
			}
		);
		crate::util_macros::assert_error!(glue,
			"SELECT LOWER(1) FROM Item",
			multisql::ValueError::CannotConvert(multisql::Value::I64(1), "TEXT")
		);
		crate::util_macros::assert_error!(glue,
			"SELECT WHATEVER(1) FROM Item",
			multisql::RecipeError::UnimplementedMethod(String::from("WHATEVER"))
		);
	})
);
