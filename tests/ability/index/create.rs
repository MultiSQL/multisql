use crate::util::*;
testcase!(test);
fn test(mut glue: multisql::Glue) {
	make_basic_table!(glue);
	assert_success!(
		glue,
		r#"
			CREATE INDEX index ON basic (a);
		"#
	);
}
