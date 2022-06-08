use crate::util::*;
testcase!(test);
fn test(mut glue: multisql::Glue) {
	assert_success!(
		glue,
		"
			CREATE TABLE simple (
				id INTEGER,
				val FLOAT
			)
		"
	);

	assert_success!(
		glue,
		"
			EXPLAIN simple
		"
	);

	assert_success!(
		glue,
		"
			EXPLAIN main
		"
	);

	assert_success!(
		glue,
		"
			EXPLAIN main.simple
		"
	);

	assert_error!(
		glue,
		"
			EXPLAIN nonsense
		"
	);

	assert_select!(glue, "
			EXPLAIN main
		" => table = Str:
		(String::from("simple"))
	);

	assert_select!(glue, "
			EXPLAIN main.simple
		" => column = Str, data_type = Str:
		(String::from("id"), String::from("Int")),
		(String::from("val"), String::from("Float"))
	);

	assert_select!(glue, "
			EXPLAIN ALL
		" => database = Str:
		(String::from("main"))
	);
	assert_select!(glue, "
			EXPLAIN ALL_TABLE
		" => database = Str, table = Str:
		(String::from("main"), String::from("simple"))
	);
}
