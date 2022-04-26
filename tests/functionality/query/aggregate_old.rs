crate::util_macros::testcase!(
	(|mut glue: multisql::Glue| {
		crate::util_macros::execute!(glue, 
			"
			CREATE TABLE Item (
				id INTEGER,
				quantity INTEGER,
				age INTEGER NULL,
			);
		"
		);
		crate::util_macros::execute!(glue, 
			"
			INSERT INTO Item (id, quantity, age) VALUES
				(1, 10, 11),
				(2, 0, 90),
				(3, 9, NULL),
				(4, 3, 3),
				(5, 25, NULL);
		"
		);
		crate::util_macros::assert_select!(glue, "SELECT COUNT(1) FROM Item" => unnamed_0 = I64: (5));
		crate::util_macros::assert_select!(glue, "SELECT count(1) FROM Item" => unnamed_0 = I64: (5));
		crate::util_macros::assert_select!(glue, "SELECT Count(1) FROM Item" => unnamed_0 = I64: (5));
		crate::util_macros::assert_select!(glue, "SELECT COUNT(1), COUNT(1) FROM Item" => unnamed_0 = I64, unnamed_1 = I64: (5, 5));
		crate::util_macros::assert_select!(glue, "SELECT COUNT(quantity) FROM Item" => unnamed_0 = I64: (5));
		// TODO: #73 crate::util_macros::assert_select!(glue, "SELECT COUNT(age) FROM Item" => unnamed_0 = I64: (3));
		crate::util_macros::assert_select!(glue, "SELECT SUM(quantity), MAX(quantity), MIN(quantity) FROM Item" => unnamed_0 = I64, unnamed_1 = I64, unnamed_2 = I64: (47, 25, 0));
		crate::util_macros::assert_select!(glue, "SELECT SUM(quantity + 1) FROM Item" => unnamed_0 = I64: (52));
		crate::util_macros::assert_select!(glue, "SELECT SUM(quantity) * 2 + MAX(quantity) - 3 / 1 FROM Item" => unnamed_0 = I64: (116));
		crate::util_macros::assert_select!(glue, "SELECT SUM(age), MAX(age), MIN(age) FROM Item" => unnamed_0 = I64, unnamed_1 = I64, unnamed_2 = I64: (104, 90, 3));
		crate::util_macros::assert_select!(glue, "SELECT SUM(age) + SUM(quantity) FROM Item" => unnamed_0 = I64: (151));
		// TODO: #73 crate::util_macros::assert_select!(glue, "SELECT COUNT(quantity) + COUNT(age) FROM Item" => unnamed_0 = I64: (8));
		crate::util_macros::assert_select!(glue, "SELECT AVG(quantity) FROM Item" => unnamed_0 = I64: (9));
		crate::util_macros::assert_select!(glue, "SELECT SUM(1 + 2) FROM Item" => unnamed_0 = I64: (15));

		crate::util_macros::assert_error!(glue,
			"SELECT SUM(id.name.ok) FROM Item;",
			multisql::RecipeError::MissingColumn(vec![
				String::from("id"),
				String::from("name"),
				String::from("ok"),
			])
		);
		crate::util_macros::assert_error!(glue,
			"SELECT SUM(num) FROM Item;",
			multisql::RecipeError::MissingColumn(vec![
				String::from("num"),
			])
		);
		crate::util_macros::execute!(glue, "DROP TABLE Item");
		crate::util_macros::execute!(glue, "
			CREATE TABLE Item (
				id INTEGER,
				quantity INTEGER NULL,
				city TEXT,
				ratio FLOAT,
			);
		");
		crate::util_macros::execute!(glue, "
			INSERT INTO Item (id, quantity, city, ratio) VALUES
				(1, 10, 'Seoul', 0.2),
				(2, 0, 'Dhaka', 0.9),
				(3, NULL, 'Beijing', 1.1),
				(3, 30, 'Daejeon', 3.2),
				(4, 11, 'Seoul', 11.1),
				(5, 24, 'Seattle', 6.11);
		");

		crate::util_macros::assert_select!(glue, "SELECT id, COUNT(1) FROM Item GROUP BY id" => id = I64, unnamed_1 = I64:
			(1, 1), (2, 1), (3, 2), (4, 1), (5, 1)
		);
		crate::util_macros::assert_select!(glue, "SELECT id FROM Item GROUP BY id" => id = I64:
			(1), (2), (3), (4), (5)
		);
		/* TODO: Null tests
		crate::util_macros::assert_select!(glue, "SELECT SUM(quantity), COUNT(1), city FROM Item GROUP BY city" => unnamed_0 = I64, unnamed_1 = I64, city = Str:
			(0, 	1, String::from("Beijing")),
			(30, 	1, String::from("Daejeon")),
			(0, 	1, String::from("Dhaka")),
			(24, 	1, String::from("Seattle")),
			(21, 	2, String::from("Seoul"))
		);*/
		crate::util_macros::assert_select!(glue, "SELECT id, city FROM Item GROUP BY city" => id = I64, city = Str:
			(3, String::from("Beijing")),
			(3, String::from("Daejeon")),
			(2, String::from("Dhaka")),
			(5, String::from("Seattle")),
			(1, String::from("Seoul"))
		);
		crate::util_macros::assert_select!(glue, "SELECT ratio FROM Item GROUP BY id, city" => ratio = F64:
			(0.2), (0.9), (1.1), (3.2), (11.1), (6.11)
		);
		crate::util_macros::assert_select!(glue, "SELECT ratio FROM Item GROUP BY id, city HAVING ratio > 10" => ratio = F64:
			(11.1)
		);
		/* TODO: #74 HAVING not working currently
		crate::util_macros::assert_select!(glue, "SELECT SUM(quantity), COUNT(1), city FROM Item GROUP BY city HAVING COUNT(1) > 1" => unnamed_0 = I64, unnamed_1 = I64, city = Str:
			(21, 2, String::from("Seoul"))
		);*/
	})
);
