use crate::util::*;
testcase!(test);
fn test(mut glue: multisql::Glue) {
	assert_select!(glue,
		"VALUES (IFNULL(NULL, 1))" => unnamed_0 = I64:
		(1)
	);
	assert_select!(glue,
		"VALUES (IFNULL(0, 1))" => unnamed_0 = I64:
		(0)
	);
	assert_select!(glue,
		"VALUES (NULLIF(0, 1))" => unnamed_0 = I64:
		(0)
	);
	assert_select!(glue,
		"VALUES (NULLIF(1, 0))" => unnamed_0 = I64:
		(1)
	);
	assert_select!(glue,
		"VALUES (NULLIF(1, 1))" => unnamed_0 = I64:
		(_)
	);
	assert_select!(glue,
		"VALUES (NULLIF(NULL, 1))" => unnamed_0 = I64:
		(_)
	);
	assert_select!(glue,
		"VALUES (NULLIF(1, NULL))" => unnamed_0 = I64:
		(1)
	);
	assert_select!(glue,
		"VALUES (NULLIF(1, 'String'))" => unnamed_0 = I64:
		(1)
	); // Should this be an error?

	assert_select!(glue,
		"VALUES (IIF(TRUE, 0, 1))" => unnamed_0 = I64:
		(0)
	);
	assert_select!(glue,
		"VALUES (IIF(FALSE, 0, 1))" => unnamed_0 = I64:
		(1)
	);
	assert_select!(glue,
		"VALUES (IIF(1=1, 0, 1))" => unnamed_0 = I64:
		(0)
	);
	assert_select!(glue,
		"VALUES (IIF(1=0, 0, 1))" => unnamed_0 = I64:
		(1)
	);
	assert_select!(glue,
		"VALUES (IIF(NULL=0, 0, 1))" => unnamed_0 = I64:
		(1)
	);
	assert_select!(glue,
		"VALUES (IIF(0=1, 'String', 1))" => unnamed_0 = I64:
		(1)
	);
	assert_select!(glue,
		"VALUES (IIF(1=1, 'String', 1))" => unnamed_0 = Str:
		(String::from("String"))
	);

	assert_select!(glue,
		"VALUES (LEN('Test'))" => unnamed_0 = I64:
		(4)
	);
	assert_select!(glue,
		"VALUES (LEN('Test test'))" => unnamed_0 = I64:
		(9)
	);
	/* TODO: #71
	assert_select!(glue,
		"VALUES (LEN(NULL))" => unnamed_0 = I64:
		(_)
	);*/

	assert_select!(glue,
		"VALUES (ROUND(1.7), ROUND(1.2), ROUND(0.9), ROUND(10000.7))" => unnamed_0 = F64, unnamed_1 = F64, unnamed_2 = F64, unnamed_3 = F64:
		(2.0, 1.0, 1.0, 10001.0)
	);

	assert_select!(glue,
		"VALUES (POW(2, 2), POW(10, 3))" => unnamed_0 = I64, unnamed_1 = I64:
		(4, 1000)
	);

	assert_select!(glue,
		"VALUES ('Hello!', REPLACE('Hello!', '!', '?'), REPLACE('Hello!!!', '!', '?'))" => unnamed_0 = Str, unnamed_1 = Str, unnamed_2 = Str:
		(String::from("Hello!"), String::from("Hello?"), String::from("Hello???"))
	);

	assert_select!(glue,
		"VALUES (CONCAT('Aee', 'Bee'), CONCAT('Aee', 'Bee', 'Cee'))" => unnamed_0 = Str, unnamed_1 = Str:
		(String::from("AeeBee"), String::from("AeeBeeCee"))
	);

	assert_success!(glue, "VALUES (CONCAT('Aee', 'Bee', 'Cee', NULL))");
	assert_success!(glue, "VALUES (UUID())");
	assert_success!(glue, "VALUES (RAND(), RAND(2, 300), RAND(-10,10))");
	assert_error!(glue, "VALUES (RAND(1))");

	glue.execute("VALUES (IIF(NULL, 0, 1))").unwrap_err(); // Should this be an error?
	glue.execute("VALUES (IIF(7, 0, 1))").unwrap_err();
	glue.execute("VALUES (LEN(100))").unwrap_err();
}
