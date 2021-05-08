macro_rules! all {
	($storage: ident) => {
		#[test]
		fn insert() {
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
		}
	};
}
pub(crate) use all;
