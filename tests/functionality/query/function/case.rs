crate::util_macros::testcase!(
	(|mut glue: multisql::Glue| {
		crate::util_macros::assert_select!(glue,
			"VALUES (CASE
				WHEN 1=0 THEN 1
				WHEN 1=1 THEN 2
				ELSE 3
			END)" => unnamed_0 = I64:
			(2)
		);
		crate::util_macros::assert_select!(glue,
			"VALUES (CASE
				WHEN 1=0 THEN 1
				WHEN 0=1 THEN 2
				ELSE 3
			END)" => unnamed_0 = I64:
			(3)
		);
		crate::util_macros::assert_select!(glue,
			"VALUES (CASE
				WHEN 1=1 THEN 1
				WHEN 0=1 THEN 2
				ELSE 3
			END)" => unnamed_0 = I64:
			(1)
		);
	})
);
