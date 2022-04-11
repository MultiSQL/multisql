crate::util_macros::testcase!(
	(|mut glue: multisql::Glue| {
		crate::util_macros::make_basic_table!(glue);

		crate::util_macros::assert_select!(glue, r#"
		WITH cte AS (
			SELECT
				a
			FROM
				basic
		)
		SELECT
			a
		FROM
			cte
	"# => a = I64: (1));

		crate::util_macros::assert_select!(glue, r#"
		WITH cte_0 AS (
			SELECT
				a
			FROM
				basic
		), cte_1 AS (
			SELECT
				a
			FROM
				cte_0
		)
		SELECT
			a
		FROM
			cte_1
	"# => a = I64: (1));

		/* TODO: #107
		glue.execute(
			r#"
		WITH cte_0 AS (
			SELECT
				a
			FROM
				cte_1
		), cte_1 AS (
			SELECT
				a
			FROM
				basic
		)
		SELECT
			a
		FROM
			cte_0
	"#,
		)
		.expect_err("CTE is not simultaneous");
		*/

		glue.execute("CREATE TABLE basic_insert (a INTEGER)")
			.unwrap();

		crate::util_macros::assert_select!(glue, r#"
		WITH basic_inserted AS (
			INSERT INTO basic_insert
			SELECT
				a
			FROM
				basic
		)
		SELECT
			a
		FROM
			basic_inserted
	"# => a = I64: (1));

		crate::util_macros::assert_select!(glue, r#"
		SELECT
			a
		FROM
			basic_insert
	"# => a = I64: (1));
	})
);
