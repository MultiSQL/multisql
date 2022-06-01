crate::util_macros::testcase!(
	(|mut glue: multisql::Glue| {
		crate::util_macros::assert_select!(glue, "VALUES (CONVERT('INTEGER', '1'))" => unnamed_0 = I64: (1));
		// TODO? crate::util_macros::assert_select!(glue, "VALUES (CONVERT(INTEGER, '1'))" => unnamed_0 = I64: (1));
		crate::util_macros::assert_select!(glue, "VALUES (CONVERT('BOOLEAN', 'true'))" => unnamed_0 = Bool: (true));
		crate::util_macros::assert_select!(glue, "VALUES (CONVERT('TIMESTAMP', '2021-04-20', 'DATE'))" => unnamed_0 = I64: (1618876800));
		crate::util_macros::assert_select!(glue, "VALUES (CONVERT('TIMESTAMP', '2021-04-20 13:20', 'DATETIME'))" => unnamed_0 = I64: (1618924800));
		crate::util_macros::assert_select!(glue, "VALUES (CONVERT('TIMESTAMP', '2021-04-20 13:20:25', 'DATETIME'))" => unnamed_0 = I64: (1618924825));
		crate::util_macros::assert_select!(glue, "VALUES (CONVERT('TIMESTAMP', '13:20', 'TIME'))" => unnamed_0 = I64: (48000));
		crate::util_macros::assert_select!(glue, "VALUES (CONVERT('TIMESTAMP', '13:20:25', 'TIME'))" => unnamed_0 = I64: (48025));
		crate::util_macros::assert_select!(glue, "VALUES (CONVERT('TIMESTAMP', '2021-04-20', 22))" => unnamed_0 = I64: (1618876800));
		crate::util_macros::assert_select!(glue, "VALUES (CONVERT('TIMESTAMP', '2021-04-20', '%Y-%m-%d'))" => unnamed_0 = I64: (1618876800));

		crate::util_macros::assert_select!(glue,
			"VALUES (CONVERT('TEXT', 10000.921, 'MONEY'), CONVERT('TEXT', 10000.921, 'SEPARATED'))" => unnamed_0 = Str, unnamed_1 = Str:
			(String::from("$10,000.92"), String::from("10,000.92"))
		);
	})
);
