crate::util_macros::testcase!((|mut glue: multisql::Glue| {
	glue.execute(
		"
        CREATE TABLE Arith (
            id INTEGER,
            num INTEGER,
            name TEXT,
        );
    "
	).expect("CREATE Arith");
	glue.execute(
		"
        INSERT INTO Arith (id, num, name) VALUES
            (1, 6, 'A'),
            (2, 8, 'B'),
            (3, 4, 'C'),
            (4, 2, 'D'),
            (5, 3, 'E');
    "
	).unwrap();

	crate::util_macros::assert_select!(glue, "SELECT COUNT(1) count FROM Arith WHERE id = 1 + 1;" => count = I64: (1));
	crate::util_macros::assert_select!(glue, "SELECT COUNT(1) count FROM Arith WHERE id < id + 1;" => count = I64: (5));
	crate::util_macros::assert_select!(glue, "SELECT COUNT(1) count FROM Arith WHERE id < num + id;" => count = I64: (5));
	crate::util_macros::assert_select!(glue, "SELECT COUNT(1) count FROM Arith WHERE id + 1 < 5;" => count = I64: (3));
	// subtract on WHERE
	crate::util_macros::assert_select!(glue, "SELECT COUNT(1) count FROM Arith WHERE id = 2 - 1;" => count = I64: (1));
	crate::util_macros::assert_select!(glue, "SELECT COUNT(1) count FROM Arith WHERE 2 - 1 = id;" => count = I64: (1));
	crate::util_macros::assert_select!(glue, "SELECT COUNT(1) count FROM Arith WHERE id > id - 1;" => count = I64: (5));
	crate::util_macros::assert_select!(glue, "SELECT COUNT(1) count FROM Arith WHERE id > id - num;" => count = I64: (5));
	crate::util_macros::assert_select!(glue, "SELECT COUNT(1) count FROM Arith WHERE 5 - id < 3;" => count = I64: (3));
	// multiply on WHERE
	crate::util_macros::assert_select!(glue, "SELECT COUNT(1) count FROM Arith WHERE id = 2 * 2;" => count = I64: (1));
	// TODO: #30 crate::util_macros::assert_select!(glue, "SELECT COUNT(1) count FROM Arith WHERE id > id * 2;" => count = I64: (0));
	// TODO: #30 crate::util_macros::assert_select!(glue, "SELECT COUNT(1) count FROM Arith WHERE id > num * id;" => count = I64: (0));
	crate::util_macros::assert_select!(glue, "SELECT COUNT(1) count FROM Arith WHERE 3 * id < 4;" => count = I64: (1));
	// divide on WHERE
	crate::util_macros::assert_select!(glue, "SELECT COUNT(1) count FROM Arith WHERE id = 5 / 2;" => count = I64: (1));
	crate::util_macros::assert_select!(glue, "SELECT COUNT(1) count FROM Arith WHERE id > id / 2;" => count = I64: (5));
	crate::util_macros::assert_select!(glue, "SELECT COUNT(1) count FROM Arith WHERE id > num / id;" => count = I64: (3));
	crate::util_macros::assert_select!(glue, "SELECT COUNT(1) count FROM Arith WHERE 10 / id = 2;" => count = I64: (2));
	// etc
	crate::util_macros::assert_select!(glue, "SELECT COUNT(1) count FROM Arith WHERE 1 + 1 = id;" => count = I64: (1));
	glue.execute("UPDATE Arith SET id = id + 1;").unwrap();
	// TODO: #30 crate::util_macros::assert_select!(glue, "SELECT COUNT(1) count FROM Arith WHERE id = 1;" => count = I64: (0));
	glue.execute("UPDATE Arith SET id = id - 1 WHERE id != 6;").unwrap();
	crate::util_macros::assert_select!(glue, "SELECT COUNT(1) count FROM Arith WHERE id <= 2;" => count = I64: (2));
	glue.execute("UPDATE Arith SET id = id * 2;").unwrap();
	glue.execute("UPDATE Arith SET id = id / 2;").unwrap();
	crate::util_macros::assert_select!(glue, "SELECT COUNT(1) count FROM Arith WHERE id <= 2;" => count = I64: (2));
}));
