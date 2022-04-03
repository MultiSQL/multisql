crate::util_macros::testcase!(
	(|mut glue: multisql::Glue| {
		crate::util_macros::execute!(glue, r#"
			CREATE TABLE basic (
				a INTEGER
			)
		"#);
		crate::util_macros::execute!(glue, r#"
			INSERT INTO basic (
				a
			) VALUES (
				1
			)
		"#);
	})
);
