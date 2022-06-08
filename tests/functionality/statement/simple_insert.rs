use crate::util::*;
testcase!(test);
fn test(mut glue: multisql::Glue) {
	execute!(
		glue,
		r#"
			CREATE TABLE basic (
				a INTEGER
			)
		"#
	);
	execute!(
		glue,
		r#"
			INSERT INTO basic (
				a
			) VALUES (
				1
			)
		"#
	);
}
