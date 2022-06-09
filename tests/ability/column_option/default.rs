use crate::util::*;
testcase!(test);
fn test(mut glue: multisql::Glue) {
	execute!(
		glue,
		"
			CREATE TABLE Test (
				id INTEGER DEFAULT 1,
				num INTEGER,
				flag BOOLEAN NULL DEFAULT false
			)
		"
	);

	execute!(
		glue,
		"
			INSERT INTO Test
			VALUES (
				8, 80, true
			);
		"
	);
	execute!(
		glue,
		"
			INSERT INTO Test (
				num
			) VALUES (
				10
			);
		"
	);
	execute!(
		glue,
		"
			INSERT INTO Test (
				num,
				id
			) VALUES (
				20,
				2
			);
		"
	);
	execute!(
		glue,
		"
			INSERT INTO Test (
				num,
				flag
			) VALUES (
				30,
				NULL
			), (
				40,
				true
			);
		"
	);

	assert_select!(glue, r#"
			SELECT
				*
			FROM
				Test
			WHERE
				flag IS NOT NULL
		"# => id = I64, num = I64, flag = Bool: (8, 80, true), (1, 10, false), (2, 20, false), /*(1, 30, NULL),*/ (1, 40, true));
}
