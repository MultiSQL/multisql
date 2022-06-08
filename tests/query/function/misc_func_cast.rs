use crate::util::*;
testcase!(test);
fn test(mut glue: multisql::Glue) {
	execute!(
		glue,
		"
			CREATE TABLE Item (
				id INTEGER NULL,
				flag BOOLEAN,
				ratio FLOAT NULL,
				number TEXT
			)
	  "
	);
	execute!(glue, "INSERT INTO Item VALUES (0, TRUE, NULL, '1')");
	assert_select!(glue, "
			SELECT CAST(LOWER(number) AS INTEGER) AS cast FROM Item
			" => cast = I64: (1)
	);
	assert_select!(glue, "
			SELECT CAST(id AS BOOLEAN) AS cast FROM Item
			" => cast = Bool: (false)
	);
	assert_select!(glue, "
			SELECT CAST(flag AS TEXT) AS cast FROM Item
			" => cast = Str: (String::from("true"))
	);
	/*(
		r#"SELECT CAST(ratio AS INTEGER) AS cast FROM Item"#,
		Ok(select_with_null!(cast; Null)),
	)
	(
		r#"SELECT CAST(number AS BOOLEAN) FROM Item"#,
		Err(ValueError::ImpossibleCast.into()),
	),
	(
		r#"SELECT CAST(number AS NULL) FROM Item"#,
		Err(ValueError::UnimplementedCast.into()),
	)*/
}
