use crate::util::*;
testcase!(test);
fn test(mut glue: multisql::Glue) {
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

	assert_select!(glue, "SELECT a FROM basic" => a = I64: (1));

	glue.execute(
		r#"
		TRUNCATE TABLE basic
	"#,
	)
	.expect("TRUNCATE basic");

	assert_select!(glue, "SELECT a FROM basic" => a = I64: );
}
