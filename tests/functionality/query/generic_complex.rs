crate::util_macros::testcase!(
	(|mut glue: multisql::Glue| {
		crate::util_macros::execute!(glue, "
			CREATE TABLE TableA (
					id INTEGER,
					test INTEGER,
					target_id INTEGER,
			);
		");
		crate::util_macros::execute!(glue, "
			INSERT INTO TableA (id, test, target_id) VALUES
				(1, 100, 2),
				(2, 100, 1),
				(3, 300, 5);
		");
		crate::util_macros::execute!(glue, "INSERT INTO TableA (target_id, id, test) VALUES (5, 3, 400);");
		crate::util_macros::execute!(glue, "INSERT INTO TableA (test, id, target_id) VALUES (500, 3, 4);");
		crate::util_macros::execute!(glue, "INSERT INTO TableA VALUES (4, 500, 3);");

		crate::util_macros::assert_select_count!(glue, "SELECT * FROM TableA;", 6);
		crate::util_macros::assert_select_count!(glue, "SELECT * FROM TableA WHERE id = 3;", 3);
		/* TODO: #50
		(3, "SELECT * FROM TableA WHERE id = (SELECT id FROM TableA WHERE id = 3 LIMIT 1)"),
		*/
		/* TODO: #49
		(3, "SELECT * FROM TableA WHERE id IN (1, 2, 4)"),
		(3, "SELECT * FROM TableA WHERE test IN (500, 300)"),
		(2, "SELECT * FROM TableA WHERE id IN (SELECT target_id FROM TableA LIMIT 3)"),
		*/
		crate::util_macros::assert_select_count!(glue, "SELECT * FROM TableA WHERE id = 3 AND test = 500;", 1);
		crate::util_macros::assert_select_count!(glue, "SELECT * FROM TableA WHERE id = 3 OR test = 100;", 5);
		crate::util_macros::assert_select_count!(glue, "SELECT * FROM TableA WHERE id != 3 AND test != 100;", 1);
		crate::util_macros::assert_select_count!(glue, "SELECT * FROM TableA WHERE id = 3 LIMIT 2;", 2);
		crate::util_macros::assert_select_count!(glue, "SELECT * FROM TableA LIMIT 10 OFFSET 2;", 4);
		crate::util_macros::assert_select_count!(glue, "SELECT * FROM TableA WHERE (id = 3 OR test = 100) AND test = 300;", 1);
		/* TODO: #50
		(4, "SELECT * FROM TableA a WHERE target_id = (SELECT id FROM TableA b WHERE b.target_id = a.id LIMIT 1);"),
		(4, "SELECT * FROM TableA a WHERE target_id = (SELECT id FROM TableA WHERE target_id = a.id LIMIT 1);"),
		*/
		crate::util_macros::assert_select_count!(glue, "SELECT * FROM TableA WHERE NOT (id = 3);", 3);

		crate::util_macros::execute!(glue, "UPDATE TableA SET test = 200 WHERE test = 100;");

		crate::util_macros::assert_select_count!(glue, "SELECT * FROM TableA WHERE test = 100;", 0);
		crate::util_macros::assert_select_count!(glue, "SELECT * FROM TableA WHERE (test = 200);", 2);
		
		crate::util_macros::execute!(glue, "DELETE FROM TableA WHERE id != 3;");
		
		crate::util_macros::assert_select_count!(glue, "SELECT * FROM TableA;", 3);
		
		crate::util_macros::execute!(glue, "DELETE FROM TableA;");

		crate::util_macros::assert_select!(glue,
			"SELECT id, test FROM TableA LIMIT 1;" => id = I64, test = I64:
			(1, 100)
		);
		crate::util_macros::assert_select!(glue,
			"SELECT id, test FROM TableA LIMIT 1;" => id = I64:
			(1)
		);
		crate::util_macros::assert_select!(glue,
			"SELECT id, test, target_id FROM TableA LIMIT 1;" => id = I64, test = I64, target_id = I64:
			(1, 100, 2)
		);
	})
);
