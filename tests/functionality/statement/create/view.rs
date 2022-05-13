crate::util_macros::testcase!(
	(|mut glue: multisql::Glue| {
		crate::util_macros::make_basic_table!(glue);
		crate::util_macros::assert_success!(
			glue,
			r#"
			CREATE VIEW basic_view AS (
                SELECT  a
                FROM    basic
            )
		"#
		);

		crate::util_macros::assert_select!(glue,
			"SELECT 1 FROM basic_view"
			=> a = I64: (1)
		);
	})
);
