use crate::*;

test_case!(convert, async move {
	let test_cases = vec![
		(
			r#"VALUES (CONVERT("INTEGER", "1"))"#,
			Ok(select!(cast Value::I64; 1)),
		),
		(
			r#"VALUES (CONVERT("BOOLEAN", "TRUE"))"#,
			Err(ValueError::UnimplementedConvert.into()),
		),
		(
			r#"VALUES (CONVERT("TIMESTAMP", "2021-04-20", "DATE"))"#,
			Ok(select!(cast Value::I64; 1618876800)),
		),
		(
			r#"VALUES (CONVERT("TIMESTAMP", "2021-04-20 13:20", "DATETIME"))"#,
			Ok(select!(cast Value::I64; 1618924800)),
		),
		(
			r#"VALUES (CONVERT("TIMESTAMP", "2021-04-20 13:20:25", "DATETIME"))"#,
			Ok(select!(cast Value::I64; 1618924825)),
		),
		(
			r#"VALUES (CONVERT("TIMESTAMP", "13:20", "TIME"))"#,
			Ok(select!(cast Value::I64; 48000)),
		),
		(
			r#"VALUES (CONVERT("TIMESTAMP", "13:20:25", "TIME"))"#,
			Ok(select!(cast Value::I64; 48025)),
		),
		(
			r#"VALUES (CONVERT("TIMESTAMP", "2021-04-20", 22))"#,
			Ok(select!(cast Value::I64; 1618876800)),
		),
		(
			r#"VALUES (CONVERT("TIMESTAMP", "2021-04-20", "%Y-%m-%d"))"#,
			Ok(select!(cast Value::I64; 1618876800)),
		),
	];
	for (sql, expected) in test_cases.into_iter() {
		test!(expected, sql);
	}
});
