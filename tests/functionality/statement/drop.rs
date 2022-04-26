crate::util_macros::testcase!(
	(|mut glue: multisql::Glue| {
		crate::util_macros::assert_success!(glue, "
			CREATE TABLE DropTable (
				id INT,
				num INT,
				name TEXT
			)
		");

		crate::util_macros::assert_success!(glue, "INSERT INTO DropTable (id, num, name) VALUES (1, 2, 'Hello')");

		crate::util_macros::assert_select_count!(glue, "SELECT id, num, name FROM DropTable;", 1);
		crate::util_macros::assert_success!(glue, "DROP TABLE DropTable;");
		crate::util_macros::assert_error!(glue,
			"DROP TABLE DropTable;",
			multisql::AlterError::TableNotFound("DropTable".to_owned())
		);
		crate::util_macros::assert_success!(glue, "
			CREATE TABLE DropTable (
				id INT,
				num INT,
				name TEXT
			)
		");
		crate::util_macros::assert_success!(glue, "DROP TABLE IF EXISTS DropTable;");
		crate::util_macros::assert_success!(glue, "DROP TABLE IF EXISTS DropTable;");
		crate::util_macros::assert_error!(glue,
			"SELECT id, num, name FROM DropTable;",
			multisql::FetchError::TableNotFound("DropTable".to_owned())
		);
		crate::util_macros::assert_success!(glue, "
			CREATE TABLE DropTable (
				id INT,
				num INT,
				name TEXT
			)
		");
		crate::util_macros::assert_select_count!(glue, "SELECT id, num, name FROM DropTable;", 0);
		crate::util_macros::assert_error!(glue,
			"DROP VIEW DropTable;",
			multisql::AlterError::DropTypeNotSupported("VIEW".to_owned())
		);
	})
);
