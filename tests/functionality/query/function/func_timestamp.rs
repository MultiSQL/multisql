crate::util_macros::testcase!(
	(|mut glue: multisql::Glue| {
		crate::util_macros::assert_success!(glue, "VALUES (NOW())");
		crate::util_macros::assert_select!(glue,
			"VALUES (CONVERT('TEXT', DATEFROMPARTS(2001,2,3), '%Y-%m-%d'))" => unnamed_0 = Str:
			(String::from("2001-02-03"))
		);
		crate::util_macros::assert_select!(glue,
			"VALUES (CONVERT('TEXT', 981158400, '%Y-%m-%d'))" => unnamed_0 = Str:
			(String::from("2001-02-03"))
		);
		crate::util_macros::assert_select!(glue,
			"VALUES (DATEFROMPARTS(2001,2,3))" => unnamed_0 = I64:
			(981158400)
		);
		/* TODO: #51
		crate::util_macros::assert_select!(glue,
			"VALUES (CAST('2001-02-03' AS TIMESTAMP))" => unnamed_0 = I64:
			(981158400)
		);*/
		crate::util_macros::assert_select!(glue,
			"VALUES (MONTH(DATEFROMPARTS(2001,2,3)))" => unnamed_0 = I64:
			(2)
		);
		crate::util_macros::assert_select!(glue,
			"VALUES (MONTH(981158400))" => unnamed_0 = I64:
			(2)
		);

		crate::util_macros::assert_select!(glue,
			"VALUES (CONVERT('TIMESTAMP', '2001-02-03 04:05:06', 'DATETIME'), DATEFROMPARTS(2001,2,3,4,5,6))" => unnamed_0 = I64, unnamed_1 = I64:
			(981173106, 981173106)
		);

		crate::util_macros::assert_select!(glue,
			"VALUES (YEAR(981173106), MONTH(981173106), DAY(981173106), HOUR(981173106), MINUTE(981173106), SECOND(981173106))" => unnamed_0 = I64, unnamed_1 = I64, unnamed_2 = I64, unnamed_3 = I64, unnamed_4 = I64, unnamed_5 = I64:
			(2001, 2, 3, 4, 5, 6)
		);

		/* TODO: #51
		crate::util_macros::assert_select!(glue,
			"VALUES (CONVERT('TIMESTAMP', '2001-02-03'))" => unnamed_0 = I64:
			(981158400)
		);*/
		crate::util_macros::assert_select!(glue,
			"VALUES (CONVERT('TIMESTAMP', '981158400', 'TIMESTAMP'))" => unnamed_0 = I64:
			(981158400)
		);
		crate::util_macros::assert_select!(glue,
			"VALUES (CONVERT('TIMESTAMP', '981158400', 0))" => unnamed_0 = I64:
			(981158400)
		);
		crate::util_macros::assert_select!(glue,
			"VALUES (CONVERT('TIMESTAMP', '2001-02-03', 'DATE'))" => unnamed_0 = I64:
			(981158400)
		);
		crate::util_macros::assert_error!(glue, "VALUES (CONVERT('TIMESTAMP', '2001-02-03', 'DATETIME'))");
		crate::util_macros::assert_select!(glue,
			"VALUES (CONVERT('TIMESTAMP', '2001-02-03 04:05', 'DATETIME'))" => unnamed_0 = I64:
			(981173100)
		);
		crate::util_macros::assert_error!(glue, "VALUES (CONVERT('TIMESTAMP', '2001-02-03', 'TIME'))");
		crate::util_macros::assert_select!(glue,
			"VALUES (CONVERT('TIMESTAMP', '04:05', 'TIME'))" => unnamed_0 = I64:
			(14700)
		);
		crate::util_macros::assert_select!(glue,
			"VALUES (CONVERT('TIMESTAMP', '04:05:00', 'TIME'))" => unnamed_0 = I64:
			(14700)
		);
		crate::util_macros::assert_select!(glue,
			"VALUES (CONVERT('TIMESTAMP', '04:05:06', 'TIME'))" => unnamed_0 = I64:
			(14706)
		);

		crate::util_macros::assert_select!(glue,
			"VALUES (CONVERT('TIMESTAMP', '03/02/2001', 'DATE'))" => unnamed_0 = I64:
			(981158400)
		);

		crate::util_macros::assert_select!(glue,
			"VALUES (CONVERT('TIMESTAMP', '13/02/2001', 'DATE'))" => unnamed_0 = I64:
			(982022400)
		);
		crate::util_macros::assert_error!(glue, "VALUES (CONVERT('TIMESTAMP', '02/13/2001', 'DATE'))");
		crate::util_macros::assert_select!(glue,
			"VALUES (CONVERT('TIMESTAMP', '03-Feb-2001', 'DATE'))" => unnamed_0 = I64:
			(981158400)
		);
		crate::util_macros::assert_select!(glue,
			"VALUES (CONVERT('TIMESTAMP', '03-Feb-01', 'DATE'))" => unnamed_0 = I64:
			(981158400)
		);
		crate::util_macros::assert_select!(glue,
			"VALUES (CONVERT('TIMESTAMP', '03-Feb-2001', 32))" => unnamed_0 = I64:
			(981158400)
		);
		crate::util_macros::assert_select!(glue,
			"VALUES (CONVERT('TIMESTAMP', '03-Feb-01', 32))" => unnamed_0 = I64:
			(-62132745600 as i64)
		);
		crate::util_macros::assert_error!(glue, "VALUES (CONVERT('TIMESTAMP', '03-Feb-2001', 33))");
		crate::util_macros::assert_select!(glue,
			"VALUES (CONVERT('TIMESTAMP', '03-Feb-01', 33))" => unnamed_0 = I64:
			(981158400)
		);

		crate::util_macros::assert_select!(glue,
			"VALUES (CONVERT('TIMESTAMP', '03/02/2001', 61))" => unnamed_0 = I64:
			(981158400)
		);
		crate::util_macros::assert_select!(glue,
			"VALUES (CONVERT('TIMESTAMP', '03/02/2001 04:05', 60))" => unnamed_0 = I64:
			(981173100)
		);
	})
);
