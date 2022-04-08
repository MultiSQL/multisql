crate::util_macros::testcase!(
	(|mut glue: multisql::Glue| {
		crate::util_macros::assert_success!(glue, "
			CREATE TABLE main.simple (
				id INTEGER,
				val FLOAT
			)
		");

		crate::util_macros::assert_error!(glue, "
			CREATE TABLE other.simple (
				id INTEGER,
				val FLOAT
			)
		");

		#[allow(unused_must_use)]
		{ // Delete file if still exists from previous test
			std::fs::remove_dir_all("data/create_test_other_database/");
		}

		crate::util_macros::assert_success!(glue, "
			CREATE DATABASE other LOCATION 'data/create_test_other_database/'
		");
		crate::util_macros::assert_error!(glue, "
			CREATE DATABASE other LOCATION 'data/create_test_other_database/'
		");
		crate::util_macros::assert_success!(glue, "
			CREATE DATABASE IF NOT EXISTS other LOCATION 'data/create_test_other_database/'
		");

		crate::util_macros::assert_success!(glue, "
			CREATE TABLE other.simple (
				id INTEGER,
				val FLOAT
			)
		");
	})
);
