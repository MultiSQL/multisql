crate::util_macros::testcase!(
	(|mut glue: multisql::Glue| {
		glue.execute(
			r#"
		CREATE TABLE basic (
			a INTEGER
		)
	"#,
		)
		.expect("CREATE TABLE basic");
	})
);
