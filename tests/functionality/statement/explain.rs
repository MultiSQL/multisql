crate::util_macros::testcase!(
	(|mut glue: multisql::Glue| {
		crate::util_macros::assert_success!(
			glue,
			"
			CREATE TABLE simple (
				id INTEGER,
				val FLOAT
			)
		"
		);

		crate::util_macros::assert_success!(
			glue,
			"
			EXPLAIN simple
		"
		);

		crate::util_macros::assert_success!(
			glue,
			"
			EXPLAIN main
		"
		);

		crate::util_macros::assert_success!(
			glue,
			"
			EXPLAIN main.simple
		"
		);

		crate::util_macros::assert_error!(
			glue,
			"
			EXPLAIN nonsense
		"
		);

		crate::util_macros::assert_select!(glue, "
			EXPLAIN main
		" => table = Str:
			(String::from("simple"))
		);

		crate::util_macros::assert_select!(glue, "
			EXPLAIN main.simple
		" => column = Str, data_type = Str:
			(String::from("id"), String::from("Int")),
			(String::from("val"), String::from("Float"))
		);

		crate::util_macros::assert_select!(glue, "
			EXPLAIN ALL
		" => database = Str:
			(String::from("main"))
		);
		crate::util_macros::assert_select!(glue, "
			EXPLAIN ALL_TABLE
		" => database = Str, table = Str:
			(String::from("main"), String::from("simple"))
		);
	})
);
