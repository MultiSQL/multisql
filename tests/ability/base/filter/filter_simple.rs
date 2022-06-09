use crate::util::*;
testcase!(test);
fn test(mut glue: multisql::Glue) {
	execute!(
		glue,
		"
		CREATE TABLE Boss (
			id INTEGER,
			name TEXT,
			strength FLOAT
		);"
	);
	execute!(
		glue,
		"
		CREATE TABLE Hunter (
			id INTEGER,
			name TEXT
		);"
	);

	execute!(
		glue,
		"
		INSERT INTO Boss (id, name, strength) VALUES
			(1, 'Amelia', 10.10),
			(2, 'Doll', 20.20),
			(3, 'Gascoigne', 30.30),
			(4, 'Gehrman', 40.40),
			(5, 'Maria', 50.50);
		"
	);
	execute!(
		glue,
		"
		INSERT INTO Hunter (id, name) VALUES
			(1, 'Gascoigne'),
			(2, 'Gehrman'),
			(3, 'Maria');
		"
	);

	assert_select_count!(
		glue,
		"SELECT id, name FROM Boss WHERE id BETWEEN 2 AND 4",
		3
	);
	assert_select_count!(
		glue,
		"SELECT id, name FROM Boss WHERE name BETWEEN 'Doll' AND 'Gehrman'",
		3
	);
	assert_select_count!(
		glue,
		"SELECT name FROM Boss WHERE name NOT BETWEEN 'Doll' AND 'Gehrman'",
		2
	);
	assert_select_count!(
		glue,
		"SELECT strength, name FROM Boss WHERE name NOT BETWEEN 'Doll' AND 'Gehrman'",
		2
	);
	// TODO: Subqueries, EXISTS
	/*(
		3,
		"SELECT name
		 FROM Boss
		 WHERE EXISTS (
			SELECT * FROM Hunter WHERE Hunter.name = Boss.name
		 )",
	),
	(
		2,
		"SELECT name
		 FROM Boss
		 WHERE NOT EXISTS (
			SELECT * FROM Hunter WHERE Hunter.name = Boss.name
		 )",
	),*/
	assert_select_count!(glue, "SELECT name FROM Boss WHERE +1 = 1", 5);
	assert_select_count!(glue, "SELECT id FROM Hunter WHERE -1 = -1", 3);
	assert_select_count!(glue, "SELECT name FROM Boss WHERE -2.0 < -1.0", 5);
	assert_select_count!(glue, "SELECT id FROM Hunter WHERE +2 > +1.0", 3);
	assert_select_count!(glue, "SELECT name FROM Boss WHERE id <= +2", 2);
	assert_select_count!(glue, "SELECT name FROM Boss WHERE +id <= 2", 2);

	assert_select_count!(glue, "SELECT name FROM Boss WHERE 2 = 1.0 + 1", 5);
	assert_select_count!(glue, "SELECT id FROM Hunter WHERE -1.0 - 1.0 < -1", 3);
	assert_select_count!(glue, "SELECT name FROM Boss WHERE -2.0 * -3.0 = 6", 5);
	assert_select_count!(glue, "SELECT id FROM Hunter WHERE +2 / 1.0 > +1.0", 3);

	assert_error!(
		glue,
		"SELECT id FROM Hunter WHERE +'abcd' > 1.0",
		multisql::ValueError::OnlySupportsNumeric(
			multisql::Value::Str(String::from("abcd")),
			"unary_plus"
		)
	);
	assert_error!(
		glue,
		"SELECT id FROM Hunter WHERE -'abcd' < 1.0",
		multisql::ValueError::OnlySupportsNumeric(
			multisql::Value::Str(String::from("abcd")),
			"unary_minus"
		)
	);
	assert_error!(
		glue,
		"SELECT id FROM Hunter WHERE +name > 1.0",
		multisql::ValueError::OnlySupportsNumeric(
			multisql::Value::Str(String::from("Gascoigne")),
			"unary_plus"
		)
	);
	assert_error!(
		glue,
		"SELECT id FROM Hunter WHERE -name < 1.0",
		multisql::ValueError::OnlySupportsNumeric(
			multisql::Value::Str(String::from("Gascoigne")),
			"unary_minus"
		)
	);
}
