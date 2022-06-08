use crate::util::*;
testcase!(test);
fn test(mut glue: multisql::Glue) {
	/* TODO: #51
	assert_select!(glue,
		"VALUES (CONVERT('TIMESTAMP', '2001-02-03'))" => unnamed_0 = I64:
		(981158400)
	);*/
	assert_select!(glue,
		"VALUES (CONVERT('TIMESTAMP', '981158400', 'TIMESTAMP'))" => unnamed_0 = I64:
		(981158400)
	);
	assert_select!(glue,
		"VALUES (CONVERT('TIMESTAMP', '981158400', 0))" => unnamed_0 = I64:
		(981158400)
	);
	assert_select!(glue,
		"VALUES (CONVERT('TIMESTAMP', '2001-02-03', 'DATE'))" => unnamed_0 = I64:
		(981158400)
	);
	assert_error!(
		glue,
		"VALUES (CONVERT('TIMESTAMP', '2001-02-03', 'DATETIME'))"
	);
	assert_select!(glue,
		"VALUES (CONVERT('TIMESTAMP', '2001-02-03 04:05', 'DATETIME'))" => unnamed_0 = I64:
		(981173100)
	);
	assert_error!(glue, "VALUES (CONVERT('TIMESTAMP', '2001-02-03', 'TIME'))");
	assert_select!(glue,
		"VALUES (CONVERT('TIMESTAMP', '04:05', 'TIME'))" => unnamed_0 = I64:
		(14700)
	);
	assert_select!(glue,
		"VALUES (CONVERT('TIMESTAMP', '04:05:00', 'TIME'))" => unnamed_0 = I64:
		(14700)
	);
	assert_select!(glue,
		"VALUES (CONVERT('TIMESTAMP', '04:05:06', 'TIME'))" => unnamed_0 = I64:
		(14706)
	);

	assert_select!(glue,
		"VALUES (CONVERT('TIMESTAMP', '03/02/2001', 'DATE'))" => unnamed_0 = I64:
		(981158400)
	);

	assert_select!(glue,
		"VALUES (CONVERT('TIMESTAMP', '13/02/2001', 'DATE'))" => unnamed_0 = I64:
		(982022400)
	);
	assert_error!(glue, "VALUES (CONVERT('TIMESTAMP', '02/13/2001', 'DATE'))");
	assert_select!(glue,
		"VALUES (CONVERT('TIMESTAMP', '03-Feb-2001', 'DATE'))" => unnamed_0 = I64:
		(981158400)
	);
	assert_select!(glue,
		"VALUES (CONVERT('TIMESTAMP', '03-Feb-01', 'DATE'))" => unnamed_0 = I64:
		(981158400)
	);
	assert_select!(glue,
		"VALUES (CONVERT('TIMESTAMP', '03-Feb-2001', 32))" => unnamed_0 = I64:
		(981158400)
	);
	assert_select!(glue,
		"VALUES (CONVERT('TIMESTAMP', '03-Feb-01', 32))" => unnamed_0 = I64:
		(-62132745600 as i64)
	);
	assert_error!(glue, "VALUES (CONVERT('TIMESTAMP', '03-Feb-2001', 33))");
	assert_select!(glue,
		"VALUES (CONVERT('TIMESTAMP', '03-Feb-01', 33))" => unnamed_0 = I64:
		(981158400)
	);

	assert_select!(glue,
		"VALUES (CONVERT('TIMESTAMP', '03/02/2001', 61))" => unnamed_0 = I64:
		(981158400)
	);
	assert_select!(glue,
		"VALUES (CONVERT('TIMESTAMP', '03/02/2001 04:05', 60))" => unnamed_0 = I64:
		(981173100)
	);
	assert_select!(glue,
		"VALUES (CAST('03/02/2001 04:05' AS TIMESTAMP))" => unnamed_0 = Timestamp:
		(981173100)
	);
}
