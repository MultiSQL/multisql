macro_rules! all {
	($storage: ident) => {
		#[test]
		fn create_table() {
			let mut glue = $storage();
			glue.execute(
				r#"
				CREATE TABLE basic (
					a INTEGER
				)
			"#,
			)
			.expect("CREATE TABLE basic");
		}
	};
}
pub(crate) use all;
