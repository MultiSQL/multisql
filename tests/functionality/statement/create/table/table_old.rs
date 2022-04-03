crate::util_macros::testcase!(
	(|mut glue: multisql::Glue| {
		crate::util_macros::assert_success!(glue, "
			CREATE TABLE CreateTable1 (
				id INTEGER NULL,
				num INTEGER,
				name TEXT
			)"
		);
		crate::util_macros::assert_error!(glue, "
			CREATE TABLE CreateTable1 (
				id INTEGER NULL,
				num INTEGER,
				name TEXT
			)",
			multisql::AlterError::TableAlreadyExists("CreateTable1".to_owned())
		);
		crate::util_macros::assert_success!(glue, "
			CREATE TABLE IF NOT EXISTS CreateTable2 (
				id INTEGER NULL,
				num INTEGER,
				name TEXT
			)"
		);
		crate::util_macros::assert_success!(glue, 
			"CREATE TABLE IF NOT EXISTS CreateTable2 (
				id2 INTEGER NULL,
				)"
		);
		crate::util_macros::assert_success!(glue, "INSERT INTO CreateTable2 VALUES (NULL, 1, '1');");
		crate::util_macros::assert_error!(glue,
			"CREATE TABLE Gluery (id SOMEWHAT);",
			multisql::AlterError::UnsupportedDataType("SOMEWHAT".to_owned())
		);
		crate::util_macros::assert_error!(glue,
			"CREATE TABLE Gluery (id INTEGER CHECK (true));",
			multisql::AlterError::UnsupportedColumnOption("CHECK (true)".to_owned())
		);
		crate::util_macros::assert_error!(glue, "
			CREATE TABLE CreateTable3 (
				id INTEGER,
				ratio FLOAT UNIQUE
			)",
			multisql::AlterError::UnsupportedDataTypeForUniqueColumn(
				"ratio".to_owned(),
				"FLOAT".to_owned(),
			)
		);
	})
);
