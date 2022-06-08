use crate::util::*;
testcase!(test);
fn test(mut glue: multisql::Glue) {
	make_basic_table!(glue);

	assert_eq!(
		glue.select_as_csv("SELECT * FROM basic"),
		Ok(String::from("a\n1\n"))
	);

	multisql::INSERT! {glue, INTO basic (a) VALUES (2),(3),(4),(5)}.unwrap();

	assert_eq!(
		glue.select_as_csv("SELECT * FROM basic ORDER BY a"),
		Ok(String::from("a\n1\n2\n3\n4\n5\n"))
	);
}
