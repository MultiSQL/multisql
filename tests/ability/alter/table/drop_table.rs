use crate::util::*;
testcase!(test);
fn test(mut glue: multisql::Glue) {
	assert_success!(
		glue,
		"
			CREATE TABLE DropTable (
				id INT,
				num INT,
				name TEXT
			)
		"
	);

	assert_success!(
		glue,
		"INSERT INTO DropTable (id, num, name) VALUES (1, 2, 'Hello')"
	);

	assert_select_count!(glue, "SELECT id, num, name FROM DropTable;", 1);
	assert_success!(glue, "DROP TABLE DropTable;");
	assert_error!(
		glue,
		"DROP TABLE DropTable;",
		multisql::AlterError::TableNotFound("DropTable".to_owned())
	);
	assert_success!(
		glue,
		"
			CREATE TABLE DropTable (
				id INT,
				num INT,
				name TEXT
			)
		"
	);
	assert_success!(glue, "DROP TABLE IF EXISTS DropTable;");
	assert_success!(glue, "DROP TABLE IF EXISTS DropTable;");
	assert_error!(
		glue,
		"SELECT id, num, name FROM DropTable;",
		multisql::FetchError::TableNotFound("DropTable".to_owned())
	);
	assert_success!(
		glue,
		"
			CREATE TABLE DropTable (
				id INT,
				num INT,
				name TEXT
			)
		"
	);
	assert_select_count!(glue, "SELECT id, num, name FROM DropTable;", 0);
	assert_error!(
		glue,
		"DROP VIEW DropTable;",
		multisql::AlterError::DropTypeNotSupported("VIEW".to_owned())
	);
}
