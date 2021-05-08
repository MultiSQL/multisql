#![allow(unused_macros)]
#![allow(unused_imports)]
macro_rules! make_basic_table {
	($glue: expr) => {
		$glue
			.execute(
				r#"
				CREATE TABLE basic (
					a INTEGER
				)
			"#,
			)
			.expect("CREATE TABLE basic");
		$glue
			.execute(
				r#"
				INSERT INTO basic (
					a
				) VALUES (
					1
				)
			"#,
			)
			.expect("INSERT basic");
	};
}
pub(crate) use make_basic_table;
