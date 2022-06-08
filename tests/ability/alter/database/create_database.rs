use crate::util::*;
testcase!(test);
fn test(mut glue: multisql::Glue) {
	assert_success!(
		glue,
		"
			CREATE TABLE main.simple (
				id INTEGER,
				val FLOAT
			)
		"
	);

	assert_error!(
		glue,
		"
			CREATE TABLE other.simple (
				id INTEGER,
				val FLOAT
			)
		"
	);

	#[allow(unused_must_use)]
	{
		// Delete file if still exists from previous test
		std::fs::remove_dir_all("data/create_test_other_database/");
	}

	assert_success!(
		glue,
		"
			CREATE DATABASE other LOCATION 'data/create_test_other_database/'
		"
	);
	assert_error!(
		glue,
		"
			CREATE DATABASE other LOCATION 'data/create_test_other_database/'
		"
	);
	assert_success!(
		glue,
		"
			CREATE DATABASE IF NOT EXISTS other LOCATION 'data/create_test_other_database/'
		"
	);

	assert_success!(
		glue,
		"
			CREATE TABLE other.simple (
				id INTEGER,
				val FLOAT
			)
		"
	);
}
