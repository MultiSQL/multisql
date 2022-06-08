use crate::util::*;
testcase!(test);
fn test(mut glue: multisql::Glue) {
	execute!(
		glue,
		"
			CREATE TABLE TestA (
				id INTEGER UNIQUE,
				num INT
			)
		"
	);

	execute!(
		glue,
		"
			CREATE TABLE TestB (
				id INTEGER UNIQUE,
				num INT UNIQUE
			)
		"
	);
	execute!(
		glue,
		"
			CREATE TABLE TestC (
				id INTEGER NULL UNIQUE,
				num INT
			)
		"
	);

	execute!(glue, "INSERT INTO TestA VALUES (1, 1)");
	execute!(glue, "INSERT INTO TestA VALUES (2, 1), (3, 1)");

	execute!(glue, "INSERT INTO TestB VALUES (1, 1)");
	execute!(glue, "INSERT INTO TestB VALUES (2, 2), (3, 3)");

	execute!(glue, "INSERT INTO TestC VALUES (NULL, 1)");
	execute!(glue, "INSERT INTO TestC VALUES (2, 2), (NULL, 3)");
	execute!(glue, "UPDATE TestC SET id = 1 WHERE num = 1");
	execute!(glue, "UPDATE TestC SET id = NULL WHERE num = 1");

	{
		let error_cases: Vec<(multisql::Error, _)> = vec![
			(
				multisql::ValidateError::DuplicateEntryOnUniqueField.into(),
				"INSERT INTO TestA VALUES (2, 2)",
			),
			(
				multisql::ValidateError::DuplicateEntryOnUniqueField.into(),
				"INSERT INTO TestA VALUES (4, 4), (4, 5)",
			),
			(
				multisql::ValidateError::DuplicateEntryOnUniqueField.into(),
				"UPDATE TestA SET id = 2 WHERE id = 1",
			),
			(
				multisql::ValidateError::DuplicateEntryOnUniqueField.into(),
				"INSERT INTO TestB VALUES (1, 3)",
			),
			(
				multisql::ValidateError::DuplicateEntryOnUniqueField.into(),
				"INSERT INTO TestB VALUES (4, 2)",
			),
			(
				multisql::ValidateError::DuplicateEntryOnUniqueField.into(),
				"INSERT INTO TestB VALUES (5, 5), (6, 5)",
			),
			(
				multisql::ValidateError::DuplicateEntryOnUniqueField.into(),
				"UPDATE TestB SET num = 2 WHERE id = 1",
			),
			(
				multisql::ValidateError::DuplicateEntryOnUniqueField.into(),
				"INSERT INTO TestC VALUES (2, 4)",
			),
			(
				multisql::ValidateError::DuplicateEntryOnUniqueField.into(),
				"INSERT INTO TestC VALUES (NULL, 5), (3, 5), (3, 6)",
			),
			(
				multisql::ValidateError::DuplicateEntryOnUniqueField.into(),
				"UPDATE TestC SET id = 1",
			),
		];

		for (_error, sql) in error_cases.into_iter() {
			assert_error!(glue, sql, _error);
		}
	}
}
