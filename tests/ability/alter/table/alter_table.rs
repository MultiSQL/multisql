use crate::util::*;
testcase!(test);
fn test(mut glue: multisql::Glue) {
	assert_success!(
		glue,
		"CREATE TABLE Foo (id INTEGER);",
		multisql::Payload::Create
	);
	assert_success!(
		glue,
		"INSERT INTO Foo VALUES (1), (2), (3);",
		multisql::Payload::Insert(3)
	);
	assert_select!(glue, "SELECT id FROM Foo" => id = I64: (1),(2),(3));

	assert_error!(glue, "ALTER TABLE Foo2 RENAME TO Bar;"); //, multisql::AlterTableError::TableNotFound("Foo2".to_owned()));
	assert_success!(
		glue,
		"ALTER TABLE Foo RENAME TO Bar;",
		multisql::Payload::AlterTable
	);
	assert_select!(glue, "SELECT id FROM Bar" => id = I64: (1),(2),(3));

	assert_success!(
		glue,
		"ALTER TABLE Bar RENAME COLUMN id TO new_id",
		multisql::Payload::AlterTable
	);
	assert_select!(glue, "SELECT new_id FROM Bar" => new_id = I64: (1),(2),(3));

	assert_error!(glue, "ALTER TABLE Bar RENAME COLUMN hello TO idid"); //, multisql::AlterTableError::RenamingColumnNotFound);
	assert_success!(
		glue,
		"CREATE TABLE Foo (id INTEGER);",
		multisql::Payload::Create
	);
	assert_success!(
		glue,
		"INSERT INTO Foo VALUES (1), (2);",
		multisql::Payload::Insert(2)
	);
	assert_select!(glue, "SELECT * FROM Foo" => id = I64: (1),(2));

	assert_error!(glue, "ALTER TABLE Foo ADD COLUMN amount INTEGER"); //, multisql::AlterTableError::DefaultValueRequired("amount INT".to_owned()));
	assert_error!(glue, "ALTER TABLE Foo ADD COLUMN id INTEGER"); //, multisql::AlterTableError::AddingColumnAlreadyExists("id".to_owned()));

	/* TODO: #72
	assert_success!(glue, "ALTER TABLE Foo ADD COLUMN amount INTEGER DEFAULT 10", multisql::Payload::AlterTable);
	assert_select!(glue, "SELECT * FROM Foo" => id = I64, amount = I64: (1, 10),(2, 10));
	assert_success!(glue, "ALTER TABLE Foo ADD COLUMN opt BOOLEAN NULL", multisql::Payload::AlterTable);
	// TODO: Null test assert_success!(glue, "SELECT * FROM Foo;", select_with_null!		pt; I64(1)   I64(10)   Null		I64(2)   I64(10)   Nu;
	assert_success!(glue, "ALTER TABLE Foo ADD COLUMN opt2 BOOLEAN NULL DEFAULT true", multisql::Payload::AlterTable);
	// TODO: Null test assert_success!(glue, "SELECT * FROM Foo;", select_with_null!		pt  | opt2; I64(1)   I64(10)   Null   Bool(true)		I64(2)   I64(10)   Null   Bool(tru;
	assert_error!(glue, "ALTER TABLE Foo ADD COLUMN something INTEGER DEFAULT (SELECT id FROM Bar LIMIT 1)", multisql::WIPError::TODO);
	*/

	assert_error!(glue, "ALTER TABLE Foo ADD COLUMN something SOMEWHAT"); //, multisql::AlterError::UnsupportedDataType("SOMEWHAT".to_owned()));
	assert_error!(
		glue,
		"ALTER TABLE Foo ADD COLUMN something INTEGER CHECK (true)"
	); //, multisql::AlterError::UnsupportedColumnOption("CHECK (true)".to_owned()));

	assert_success!(
		glue,
		"ALTER TABLE Foo ADD COLUMN something FLOAT UNIQUE",
		multisql::Payload::AlterTable
	);
	assert_success!(
		glue,
		"ALTER TABLE Foo DROP COLUMN IF EXISTS something;",
		multisql::Payload::AlterTable
	);
	assert_error!(glue, "ALTER TABLE Foo DROP COLUMN something;"); //, multisql::AlterTableError::DroppingColumnNotFound("something".to_owned()));
	assert_success!(
		glue,
		"ALTER TABLE Foo DROP COLUMN amount;",
		multisql::Payload::AlterTable
	);
	// TODO: Null test assert_success!(glue, "SELECT * FROM Foo;", select_with_null!		; I64(1)   Null   Bool(true)		I64(2)   Null   Bool(tru;
	assert_success!(
		glue,
		"ALTER TABLE Foo DROP COLUMN IF EXISTS opt2;",
		multisql::Payload::AlterTable
	);
	// TODO: Null test assert_success!(glue, "SELECT * FROM Foo;", select_with_null!			id     | opt;I64(1)   Null; I64(2)   Nu;
}
