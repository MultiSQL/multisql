crate::util_macros::testcase!(
	(|mut glue: multisql::Glue| {

		crate::util_macros::execute!(glue, "
			CREATE TABLE Test (
				id INTEGER DEFAULT 1,
				num INTEGER,
				flag BOOLEAN NULL DEFAULT false
			)
		");

		crate::util_macros::execute!(glue, "
			INSERT INTO Test
			VALUES (
				8, 80, true
			);
		");
		crate::util_macros::execute!(glue, "
			INSERT INTO Test (
				num
			) VALUES (
				10
			);
		");
		crate::util_macros::execute!(glue, "
			INSERT INTO Test (
				num,
				id
			) VALUES (
				20,
				2
			);
		");
		crate::util_macros::execute!(glue, "
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
		");

		crate::util_macros::assert_select!(glue, r#"
			SELECT
				*
			FROM
				Test
			WHERE
				flag IS NOT NULL
		"# => id = I64, num = I64, flag = Bool: (8, 80, true), (1, 10, false), (2, 20, false), /*(1, 30, NULL),*/ (1, 40, true));
	})
);
