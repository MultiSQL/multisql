use crate::util::*;
testcase!(test);
fn test(mut glue: multisql::Glue) {
	make_basic_table!(glue);
	assert_success!(
		glue,
		r#"
			CREATE VIEW basic_view AS (
                SELECT  a
                FROM    basic
            )
		"#
	);

	assert_select!(glue,
		"SELECT a FROM basic_view"
		=> a = I64: (1)
	);
}
