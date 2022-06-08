use crate::util::*;
testcase!(test);
fn test(mut glue: multisql::Glue) {
	let basic_table = |num: u16| {
		use fstrings::*;
		f!(r#"
			CREATE TABLE basic_{num} (
				id INTEGER,
				num INTEGER,
				name TEXT
			)
		"#)
	};
	glue.execute(&basic_table(0)).unwrap();
	glue.execute(&basic_table(1)).unwrap();

	glue.execute("INSERT INTO basic_0 (id, num, name) VALUES (1, 2, 'Hello')")
		.unwrap();
	glue.execute("INSERT INTO basic_0 (id, num, name) VALUES (1, 9, 'World')")
		.unwrap();
	glue.execute("INSERT INTO basic_0 (id, num, name) VALUES (3, 4, 'Great'), (4, 7, 'Job')")
		.unwrap();
	glue.execute("INSERT INTO basic_1 (id, num, name) SELECT id, num, name FROM basic_0")
		.unwrap();

	glue.execute("CREATE TABLE basic_a (id INTEGER);").unwrap();
	glue.execute("INSERT INTO basic_a (id) SELECT id FROM basic_0")
		.unwrap();

	assert_select!(glue, "SELECT * FROM basic_a" => id = I64: (1), (1), (3), (4));

	assert_select!(glue,
		"SELECT id, num, name FROM basic_0" =>
		id = I64, num = I64, name = Str:
		(1, 2, String::from("Hello")),
		(1, 9, String::from("World")),
		(3, 4, String::from("Great")),
		(4, 7, String::from("Job"))
	);

	assert_select!(glue,
		"SELECT id, num, name FROM basic_1" =>
		id = I64, num = I64, name = Str:
		(1, 2, String::from("Hello")),
		(1, 9, String::from("World")),
		(3, 4, String::from("Great")),
		(4, 7, String::from("Job"))
	);

	glue.execute("UPDATE basic_0 SET id = 2").unwrap();

	assert_select!(glue, "SELECT id FROM basic_0" => id = I64: (2), (2), (2), (2));
	assert_select!(glue, "SELECT id, num FROM basic_0" => id = I64, num = I64: (2, 2), (2, 9), (2, 4), (2, 7));
}
