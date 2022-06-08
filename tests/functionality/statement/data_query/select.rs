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
	glue.execute(
		r#"
		SELECT
			a
		FROM
			basic
	"#,
	)
	.expect("SELECT basic");
}
