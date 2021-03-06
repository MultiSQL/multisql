use crate::util::*;
testcase!(test);
fn test(mut glue: multisql::Glue) {
	assert!(matches!(
		glue.execute(
			r#"
					CREATE TABLE indexed (
						a INTEGER
					)
				"#
		),
		Ok(_)
	));

	assert!(matches!(
		glue.execute(
			r#"
					INSERT INTO indexed (
						a
					) VALUES (
						1
					), (
						2
					), (
						3
					), (
						3
					), (
						4
					), (
						100
					)

				"#
		),
		Ok(_)
	));

	assert_select!(glue, r#"
			SELECT
				a
			FROM
				indexed
		"# => a = I64: (1),(2),(3),(3),(4),(100));
	assert_select!(glue, r#"
			SELECT
				a
			FROM
				indexed
			WHERE
				a > 2
		"# => a = I64: (3),(3),(4),(100));
	assert_select!(glue, r#"
			SELECT
				a
			FROM
				indexed
			WHERE
				a < 4
		"# => a = I64: (1),(2),(3),(3));

	assert!(matches!(
		glue.execute(
			r#"
					CREATE INDEX index ON indexed (a)
				"#
		),
		Ok(_)
	));

	assert_select!(glue, r#"
			SELECT
				a
			FROM
				indexed
		"# => a = I64: (1),(2),(3),(3),(4),(100));
	assert_select!(glue, r#"
			SELECT
				a
			FROM
				indexed
			WHERE
				a >= 3
		"# => a = I64: (3),(3),(4),(100));
	assert_select!(glue, r#"
			SELECT
				a
			FROM
				indexed
			WHERE
				a > 2
		"# => a = I64: (3),(3),(4),(100));
	assert_select!(glue, r#"
			SELECT
				a
			FROM
				indexed
			WHERE
				a <= 3
		"# => a = I64: (1),(2),(3),(3));
	assert_select!(glue, r#"
			SELECT
				a
			FROM
				indexed
			WHERE
				a < 4
		"# => a = I64: (1),(2),(3),(3));

	assert_select!(glue, r#"
			SELECT
				a
			FROM
				indexed
			WHERE
				a > 1 + 1
		"# => a = I64: (3),(3),(4),(100));
	assert_select!(glue, r#"
			SELECT
				a
			FROM
				indexed
			WHERE
				1 + a < 4
		"# => a = I64: (1),(2));
	assert_select!(glue, r#"
			SELECT
				a
			FROM
				indexed
			WHERE
				a < a + 1
		"# => a = I64: (1),(2),(3),(3),(4),(100));
	assert_select!(glue, r#"
			SELECT
				a
			FROM
				indexed
			WHERE
				a > a + 1
		"# => a = I64: );

	assert_select!(glue, r#"
			SELECT
				a
			FROM
				indexed
			WHERE
				a < 4
				AND a < 4
		"# => a = I64: (1),(2),(3),(3));
	assert_select!(glue, r#"
			SELECT
				a
			FROM
				indexed
			WHERE
				a < 4
				AND a > 1
		"# => a = I64: (2),(3),(3));

	assert!(matches!(
		glue.execute(
			r#"
					INSERT INTO indexed (
						a
					) VALUES (
						1
					), (
						10
					)
				"#
		),
		Ok(_)
	));

	assert_select!(glue, r#"
			SELECT
				a
			FROM
				indexed
			WHERE
				a > 2
		"# => a = I64: (3),(3),(4),(10),(100));
	assert_select!(glue, r#"
			SELECT
				a
			FROM
				indexed
			WHERE
				a < 2
		"# => a = I64: (1),(1));

	assert!(matches!(
		glue.execute(
			r#"
					INSERT INTO indexed (
						a
					) VALUES (
						-5
					)
				"#
		),
		Ok(_)
	));

	assert_select!(glue, r#"
			SELECT
				a
			FROM
				indexed
			WHERE
				a > 2
		"# => a = I64: (3),(3),(4),(10),(100));
	assert_select!(glue, r#"
			SELECT
				a
			FROM
				indexed
			WHERE
				a < 2
		"# => a = I64: (-5),(1),(1));

	assert!(matches!(
		glue.execute(
			r#"
					DELETE FROM indexed
					WHERE
						a = 2
						OR a = 4
				"#
		),
		Ok(_)
	));

	assert_select!(glue, r#"
			SELECT
				a
			FROM
				indexed
			WHERE
				a > 2
		"# => a = I64: (3),(3),(10),(100));

	assert!(matches!(
		glue.execute(
			r#"
					UPDATE indexed
					SET
						a = -100
					WHERE
						a = 100
				"#
		),
		Ok(_)
	));

	assert_select!(glue, r#"
			SELECT
				a
			FROM
				indexed
			WHERE
				a > 2
		"# => a = I64: (3),(3),(10));
}
