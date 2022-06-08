use crate::util::*;
testcase!(test);
fn test(mut glue: multisql::Glue) {
	execute!(glue, "CREATE TABLE TableA (id INTEGER);");
	execute!(glue, "INSERT INTO TableA (id) VALUES (1);");

	assert_error!(glue, "COMMIT;", multisql::ExecuteError::QueryNotSupported);
	assert_error!(
		glue,
		"INSERT INTO Nothing VALUES (1);",
		multisql::ExecuteError::TableNotExists
	);
	assert_error!(
		glue,
		"UPDATE Nothing SET a = 1;",
		multisql::ExecuteError::TableNotExists
	);
	assert_error!(
		glue,
		"SELECT * FROM Nothing;",
		multisql::FetchError::TableNotFound(String::from("Nothing"))
	);
	assert_error!(
		glue,
		"SELECT * FROM TableA JOIN (SELECT * FROM TableB) as TableC ON 1 = 1",
		multisql::JoinError::UnimplementedTableType
	);
	assert_error!(
		glue,
		"SELECT * FROM TableA WHERE id = (SELECT id FROM TableA WHERE id = 2);",
		multisql::ManualError::UnimplementedSubquery
	);
	assert_error!(
		glue,
		"SELECT * FROM TableA WHERE noname = 1;",
		multisql::RecipeError::MissingColumn(vec![String::from("noname")])
	);
	assert_error!(
		glue,
		"INSERT INTO TableA (id2) VALUES (1);",
		multisql::ValidateError::ColumnNotFound(String::from("id2"))
	);
	assert_error!(
		glue,
		"INSERT INTO TableA (id2, id) VALUES (100);",
		multisql::ValidateError::ColumnNotFound(String::from("id2"))
	);
	assert_error!(
		glue,
		"INSERT INTO TableA VALUES (100), (100, 200);",
		multisql::ValidateError::WrongNumberOfValues
	);
	assert_error!(
		glue,
		"SELECT * FROM TableA Where id = X'123';",
		multisql::ValueError::UnimplementedLiteralType
	);
}
