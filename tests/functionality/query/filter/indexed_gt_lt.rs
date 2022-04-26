crate::util_macros::testcase!(
	(|mut glue: multisql::Glue| {
		crate::util_macros::execute!(glue, 
			"CREATE TABLE Operator (
				id INTEGER,
				name TEXT,
			);"
		);
		crate::util_macros::execute!(glue, 
			"INSERT INTO Operator (id, name) VALUES
				(1, 'Abstract'),
				(2, 'Azzzz'),
				(3, 'July'),
				(4, 'Romeo'),
				(5, 'Trade');"
		);
		crate::util_macros::execute!(glue, "CREATE INDEX Operator_id ON Operator (id)");
		crate::util_macros::execute!(glue, "CREATE INDEX Operator_name ON Operator (name)");



		crate::util_macros::assert_select_count!(glue, "SELECT * FROM Operator WHERE id < 2;", 1);
		crate::util_macros::assert_select_count!(glue, "SELECT * FROM Operator WHERE id <= 2;", 2);
		crate::util_macros::assert_select_count!(glue, "SELECT * FROM Operator WHERE id > 2;", 3);
		crate::util_macros::assert_select_count!(glue, "SELECT * FROM Operator WHERE id >= 2;", 4);
		crate::util_macros::assert_select_count!(glue, "SELECT * FROM Operator WHERE 2 > id;", 1);
		crate::util_macros::assert_select_count!(glue, "SELECT * FROM Operator WHERE 2 >= id;", 2);
		crate::util_macros::assert_select_count!(glue, "SELECT * FROM Operator WHERE 2 < id;", 3);
		crate::util_macros::assert_select_count!(glue, "SELECT * FROM Operator WHERE 2 <= id;", 4);
		crate::util_macros::assert_select_count!(glue, "SELECT * FROM Operator WHERE 1 < 3;", 5);
		crate::util_macros::assert_select_count!(glue, "SELECT * FROM Operator WHERE 3 >= 3;", 5);
		crate::util_macros::assert_select_count!(glue, "SELECT * FROM Operator WHERE 3 > 3;", 0);
		/* TODO: #50
		crate::util_macros::assert_select_count!(glue, "SELECT * FROM Operator o1 WHERE 3 > (SELECT id FROM Operator WHERE o1.id < 100);", 5);
		*/
		crate::util_macros::assert_select_count!(glue, "SELECT * FROM Operator WHERE name < 'Azzzzzzzzzz';", 2);
		crate::util_macros::assert_select_count!(glue, "SELECT * FROM Operator WHERE name < 'Az';", 1);
		crate::util_macros::assert_select_count!(glue, "SELECT * FROM Operator WHERE name < 'zz';", 5);
		crate::util_macros::assert_select_count!(glue, "SELECT * FROM Operator WHERE 'aa' < 'zz';", 5);
		crate::util_macros::assert_select_count!(glue, "SELECT * FROM Operator WHERE 'Romeo' >= name;", 4);
		/* TODO: #50
		crate::util_macros::assert_select_count!(glue, "SELECT * FROM Operator WHERE (SELECT name FROM Operator LIMIT 1) >= name", 1);
		crate::util_macros::assert_select_count!(glue, "SELECT * FROM Operator WHERE name <= (SELECT name FROM Operator LIMIT 1)", 1);
		crate::util_macros::assert_select_count!(glue, "SELECT * FROM Operator WHERE 'zz' > (SELECT name FROM Operator LIMIT 1)", 5);
		crate::util_macros::assert_select_count!(glue, "SELECT * FROM Operator WHERE (SELECT name FROM Operator LIMIT 1) < 'zz'", 5);
		*/
		crate::util_macros::assert_select_count!(glue, "SELECT * FROM Operator WHERE NOT (1 != 1);", 5);
	})
);
