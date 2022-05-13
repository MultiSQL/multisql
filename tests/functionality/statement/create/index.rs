crate::util_macros::testcase!(
	(|mut glue: multisql::Glue| {
		crate::util_macros::make_basic_table!(glue);
		crate::util_macros::assert_success!(glue, r#"
			CREATE INDEX index ON basic (a);
		"#);
	})
);
