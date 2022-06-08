use crate::util::*;
testcase!(test);
fn test(mut glue: multisql::Glue) {
	assert_success!(
		glue,
		"CREATE TABLE test (
				a INTEGER NULL,
				b TEXT NULL,
			)"
	);
	assert_success!(
		glue,
		"CREATE TABLE select_into (
				x INTEGER NULL,
				y TEXT NULL,
			)"
	);
	assert_success!(
		glue,
		"INSERT INTO select_into (x, y) VALUES (10, 'j')",
		multisql::Payload::Insert(1)
	);
	assert_success!(
		glue,
		"INSERT INTO test VALUES (1, 'a');",
		multisql::Payload::Insert(1)
	);
	assert_success!(
		glue,
		"INSERT INTO test (a, b) VALUES (2, 'b');",
		multisql::Payload::Insert(1)
	);
	assert_success!(
		glue,
		"INSERT INTO test (a) VALUES (3);",
		multisql::Payload::Insert(1)
	);
	assert_success!(
		glue,
		"INSERT INTO test (b) VALUES ('c');",
		multisql::Payload::Insert(1)
	);
	assert_success!(
		glue,
		"INSERT INTO test SELECT * FROM select_into;",
		multisql::Payload::Insert(1)
	);
	assert_success!(
		glue,
		"INSERT INTO test (a, b) SELECT * FROM select_into;",
		multisql::Payload::Insert(1)
	);
	assert_success!(
		glue,
		"INSERT INTO test SELECT x, y FROM select_into;",
		multisql::Payload::Insert(1)
	);
	assert_success!(
		glue,
		"INSERT INTO test (a, b) SELECT x, y FROM select_into;",
		multisql::Payload::Insert(1)
	);
	assert_success!(
		glue,
		"INSERT INTO test (a) SELECT x FROM select_into;",
		multisql::Payload::Insert(1)
	);
	assert_success!(
		glue,
		"INSERT INTO test (b) VALUES (UPPER('test'));",
		multisql::Payload::Insert(1)
	);
	assert_success!(
		glue,
		"INSERT INTO test (b) SELECT UPPER('test') FROM select_into;",
		multisql::Payload::Insert(1)
	);

	assert_error!(
		glue,
		"INSERT INTO test (a, b) VALUES (1, 'error', 'error')",
		multisql::ValidateError::WrongNumberOfValues
	);
	assert_error!(
		glue,
		"INSERT INTO test (a, b) VALUES (1, 'error'), (1, 'error', 'error')",
		multisql::ValidateError::WrongNumberOfValues
	);
}
