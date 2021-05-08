macro_rules! all {
	($storage: ident) => {
		#[test]
		fn truncate() {
			let mut glue = $storage();
			glue.execute(
				r#"
				CREATE TABLE basic (
					a INTEGER
				)
			"#,
			)
			.expect("CREATE TABLE basic");
			glue.execute(
				r#"
				INSERT INTO basic (
					a
				) VALUES (
					1
				)
			"#,
			)
			.expect("INSERT basic");

			crate::util_macros::assert_select!(glue, "SELECT a FROM basic" => a = I64: (1));

			glue.execute(
				r#"
				TRUNCATE TABLE basic
			"#,
			)
			.expect("TRUNCATE basic");

			crate::util_macros::assert_select!(glue, "SELECT a FROM basic" => a = I64: );
		}
	};
}
pub(crate) use all;
