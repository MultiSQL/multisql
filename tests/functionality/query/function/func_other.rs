crate::util_macros::testcase!(
	(|mut glue: multisql::Glue| {
		crate::util_macros::assert_select!(glue,
			"VALUES (IFNULL(NULL, 1))" => unnamed_0 = I64:
			(1)
		);
		crate::util_macros::assert_select!(glue,
			"VALUES (IFNULL(0, 1))" => unnamed_0 = I64:
			(0)
		);
		crate::util_macros::assert_select!(glue,
			"VALUES (NULLIF(0, 1))" => unnamed_0 = I64:
			(0)
		);
		crate::util_macros::assert_select!(glue,
			"VALUES (NULLIF(1, 0))" => unnamed_0 = I64:
			(1)
		);
		crate::util_macros::assert_select!(glue,
			"VALUES (NULLIF(1, 1))" => unnamed_0 = I64:
			(_)
		);
		crate::util_macros::assert_select!(glue,
			"VALUES (NULLIF(NULL, 1))" => unnamed_0 = I64:
			(_)
		);
		crate::util_macros::assert_select!(glue,
			"VALUES (NULLIF(1, NULL))" => unnamed_0 = I64:
			(1)
		);
		crate::util_macros::assert_select!(glue,
			"VALUES (NULLIF(1, 'String'))" => unnamed_0 = I64:
			(1)
		); // Should this be an error?

		crate::util_macros::assert_select!(glue,
			"VALUES (IIF(TRUE, 0, 1))" => unnamed_0 = I64:
			(0)
		);
		crate::util_macros::assert_select!(glue,
			"VALUES (IIF(FALSE, 0, 1))" => unnamed_0 = I64:
			(1)
		);
		crate::util_macros::assert_select!(glue,
			"VALUES (IIF(1=1, 0, 1))" => unnamed_0 = I64:
			(0)
		);
		crate::util_macros::assert_select!(glue,
			"VALUES (IIF(1=0, 0, 1))" => unnamed_0 = I64:
			(1)
		);
		crate::util_macros::assert_select!(glue,
			"VALUES (IIF(NULL=0, 0, 1))" => unnamed_0 = I64:
			(1)
		);
		crate::util_macros::assert_select!(glue,
			"VALUES (IIF(0=1, 'String', 1))" => unnamed_0 = I64:
			(1)
		);
		crate::util_macros::assert_select!(glue,
			"VALUES (IIF(1=1, 'String', 1))" => unnamed_0 = Str:
			(String::from("String"))
		);

		crate::util_macros::assert_select!(glue,
			"VALUES (LEN('Test'))" => unnamed_0 = I64:
			(4)
		);
		crate::util_macros::assert_select!(glue,
			"VALUES (LEN('Test test'))" => unnamed_0 = I64:
			(9)
		);
		/* TODO: #71
		crate::util_macros::assert_select!(glue,
			"VALUES (LEN(NULL))" => unnamed_0 = I64:
			(_)
		);*/

		glue.execute("VALUES (IIF(NULL, 0, 1))").unwrap_err(); // Should this be an error?
		glue.execute("VALUES (IIF(7, 0, 1))").unwrap_err();
		glue.execute("VALUES (LEN(100))").unwrap_err();
	})
);
