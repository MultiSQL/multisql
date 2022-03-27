crate::util_macros::testcase!(
	(|mut glue: multisql::Glue| {
		crate::util_macros::make_basic_table!(glue);
		glue.execute(r#"
			CREATE INDEX index ON basic (a);
		"#).expect("CREATE INDEX basic_index");
	})
);
