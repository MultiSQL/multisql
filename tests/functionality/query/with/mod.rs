macro_rules! all {
	($storage: ident) => {
		#[test]
		fn with() {
			let mut glue = $storage();
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

			glue.execute(r#"
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
			"#).expect_err("CTE is not simultaneous");
		}
	};
}
pub(crate) use all;
