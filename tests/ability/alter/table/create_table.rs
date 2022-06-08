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

	assert_success!(
		glue,
		"
			CREATE TABLE CreateTable1 (
				id INTEGER NULL,
				num INTEGER,
				name TEXT
			)"
	);
	assert_error!(
		glue,
		"
			CREATE TABLE CreateTable1 (
				id INTEGER NULL,
				num INTEGER,
				name TEXT
			)",
		multisql::AlterError::TableAlreadyExists("CreateTable1".to_owned())
	);
	assert_success!(
		glue,
		"
			CREATE TABLE IF NOT EXISTS CreateTable2 (
				id INTEGER NULL,
				num INTEGER,
				name TEXT
			)"
	);
	assert_success!(
		glue,
		"CREATE TABLE IF NOT EXISTS CreateTable2 (
				id2 INTEGER NULL,
				)"
	);
	assert_success!(glue, "INSERT INTO CreateTable2 VALUES (NULL, 1, '1');");
	assert_error!(
		glue,
		"CREATE TABLE Gluery (id SOMEWHAT);",
		multisql::AlterError::UnsupportedDataType("SOMEWHAT".to_owned())
	);
	assert_error!(
		glue,
		"CREATE TABLE Gluery (id INTEGER CHECK (true));",
		multisql::AlterError::UnsupportedColumnOption("CHECK (true)".to_owned())
	);
	assert_error!(
		glue,
		"
			CREATE TABLE CreateTable3 (
				id INTEGER,
				ratio FLOAT UNIQUE
			)",
		multisql::AlterError::UnsupportedDataTypeForUniqueColumn(
			"ratio".to_owned(),
			"FLOAT".to_owned(),
		)
	);
}
