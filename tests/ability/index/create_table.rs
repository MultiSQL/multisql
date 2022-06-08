use crate::util::*;
testcase!(test);
fn test(mut glue: multisql::Glue) {
	glue.execute(
		r#"
			CREATE TABLE basic_keyed (
				a INTEGER PRIMARY KEY
			)
		"#,
	)
	.expect("CREATE TABLE basic_keyed");

	/*glue.execute(
		r#"
			CREATE TABLE basic_indexed_a (
				a INTEGER INDEX basic_index_a
			)
		"#,
	)
	.expect("CREATE TABLE basic_indexed_a");*/

	glue.execute(
		r#"
			CREATE TABLE basic_indexed_b (
				a INTEGER,
				INDEX basic_index_b (a)
			)
		"#,
	)
	.expect("CREATE TABLE basic_indexed_b");
}
