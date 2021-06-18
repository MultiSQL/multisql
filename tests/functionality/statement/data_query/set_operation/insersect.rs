crate::util_macros::testcase!(
	(|mut glue: multisql::Glue| {
		crate::util_macros::assert_select!(
			glue,
			r#"
				VALUES (
					'Test',
					1
				), (
					'Test2',
					2
				), (
					'Test3',
					3
				), (
					'Test4',
					4
				)
				INTERSECT
				VALUES (
					'Test3',
					3
				), (
					'Test1',
					1
				), (
					'Test5',
					5
				), (
					'Test2',
					2
				)
			"# =>
				unnamed_0 = Str, unnamed_1 = I64:
				("Test3", 3), ("Test2", 2)
		);
	})
);
