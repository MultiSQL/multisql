use crate::util::*;
testcase!(test);
fn test(mut glue: multisql::Glue) {
	make_basic_table!(glue);

	assert_select!(glue, r#"
			SELECT
				SUM(a) AS agg
			FROM
				basic
		"# => agg = I64: (1));

	assert_select!(glue, r#"
			SELECT
				COUNT(a) AS agg
			FROM
				basic
		"# => agg = I64: (1));

	assert_select!(glue, r#"
			SELECT
				COUNT(1) AS agg
			FROM
				basic
		"# => agg = I64: (1));

	assert_select!(glue, r#"
			SELECT
				COUNT(1) AS agg
			FROM
				basic
			GROUP BY
				a
		"# => agg = I64: (1));

	assert_select!(glue, r#"
			SELECT
				COUNT(1) AS agg
			FROM
				basic
			HAVING
				a = 1
		"# => agg = I64: (1));

	{
		let _expect = multisql::Payload::Select {
			labels: vec![String::from("agg")],
			rows: vec![multisql::Row(vec![multisql::Value::Null])],
		};
		assert!(matches!(
			glue.execute(
				r#"
						SELECT
							COUNT(1) AS agg
						FROM
							basic
						HAVING
							a = 0
					"#
			),
			Ok(_expect)
		));
	}

	glue.execute(
		r#"
			INSERT INTO basic (
				a
			) VALUES (
				2
			), (
				3
			)
		"#,
	)
	.unwrap();

	assert_select!(glue, r#"
			SELECT
				COUNT(1) AS agg
			FROM
				basic
		"# => agg = I64: (3));

	assert_select!(glue, r#"
			SELECT
				COUNT(a) AS agg
			FROM
				basic
		"# => agg = I64: (3));

	assert_select!(glue, r#"
			SELECT
				SUM(a) AS sum,
				MIN(a) AS min,
				MAX(a) AS max,
				AVG(a) AS avg
			FROM
				basic
		"# => sum = I64, min = I64, max = I64, avg = I64: (6, 1, 3, 2));

	assert_select!(glue, r#"
			SELECT
				a
			FROM
				basic
			GROUP BY
				a
		"# => a = I64: (1),(2),(3));
}
