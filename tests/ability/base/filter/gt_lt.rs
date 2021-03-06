use crate::util::*;
testcase!(test);
fn test(mut glue: multisql::Glue) {
	execute!(
		glue,
		"CREATE TABLE Operator (
			id INTEGER,
			name TEXT,
		);"
	);
	execute!(
		glue,
		"INSERT INTO Operator (id, name) VALUES
			(1, 'Abstract'),
			(2, 'Azzzz'),
			(3, 'July'),
			(4, 'Romeo'),
			(5, 'Trade');"
	);

	assert_select_count!(glue, "SELECT * FROM Operator WHERE id < 2;", 1);
	assert_select_count!(glue, "SELECT * FROM Operator WHERE id <= 2;", 2);
	assert_select_count!(glue, "SELECT * FROM Operator WHERE id > 2;", 3);
	assert_select_count!(glue, "SELECT * FROM Operator WHERE id >= 2;", 4);
	assert_select_count!(glue, "SELECT * FROM Operator WHERE 2 > id;", 1);
	assert_select_count!(glue, "SELECT * FROM Operator WHERE 2 >= id;", 2);
	assert_select_count!(glue, "SELECT * FROM Operator WHERE 2 < id;", 3);
	assert_select_count!(glue, "SELECT * FROM Operator WHERE 2 <= id;", 4);
	assert_select_count!(glue, "SELECT * FROM Operator WHERE 1 < 3;", 5);
	assert_select_count!(glue, "SELECT * FROM Operator WHERE 3 >= 3;", 5);
	assert_select_count!(glue, "SELECT * FROM Operator WHERE 3 > 3;", 0);
	/* TODO: #50
	assert_select_count!(glue, "SELECT * FROM Operator o1 WHERE 3 > (SELECT id FROM Operator WHERE o1.id < 100);", 5);
	*/
	assert_select_count!(
		glue,
		"SELECT * FROM Operator WHERE name < 'Azzzzzzzzzz';",
		2
	);
	assert_select_count!(glue, "SELECT * FROM Operator WHERE name < 'Az';", 1);
	assert_select_count!(glue, "SELECT * FROM Operator WHERE name < 'zz';", 5);
	assert_select_count!(glue, "SELECT * FROM Operator WHERE 'aa' < 'zz';", 5);
	assert_select_count!(glue, "SELECT * FROM Operator WHERE 'Romeo' >= name;", 4);
	/* TODO: #50
	assert_select_count!(glue, "SELECT * FROM Operator WHERE (SELECT name FROM Operator LIMIT 1) >= name", 1);
	assert_select_count!(glue, "SELECT * FROM Operator WHERE name <= (SELECT name FROM Operator LIMIT 1)", 1);
	assert_select_count!(glue, "SELECT * FROM Operator WHERE 'zz' > (SELECT name FROM Operator LIMIT 1)", 5);
	assert_select_count!(glue, "SELECT * FROM Operator WHERE (SELECT name FROM Operator LIMIT 1) < 'zz'", 5);
	*/
	assert_select_count!(glue, "SELECT * FROM Operator WHERE NOT (1 != 1);", 5);
}
