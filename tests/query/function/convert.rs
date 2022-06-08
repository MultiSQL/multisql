use crate::util::*;
testcase!(test);
fn test(mut glue: multisql::Glue) {
	assert_select!(glue, "VALUES (CONVERT('INTEGER', '1'))" => unnamed_0 = I64: (1));
	// TODO? assert_select!(glue, "VALUES (CONVERT(INTEGER, '1'))" => unnamed_0 = I64: (1));
	assert_select!(glue, "VALUES (CONVERT('BOOLEAN', 'true'))" => unnamed_0 = Bool: (true));
	assert_select!(glue, "VALUES (CONVERT('TIMESTAMP', '2021-04-20', 'DATE'))" => unnamed_0 = I64: (1618876800));
	assert_select!(glue, "VALUES (CONVERT('TIMESTAMP', '2021-04-20 13:20', 'DATETIME'))" => unnamed_0 = I64: (1618924800));
	assert_select!(glue, "VALUES (CONVERT('TIMESTAMP', '2021-04-20 13:20:25', 'DATETIME'))" => unnamed_0 = I64: (1618924825));
	assert_select!(glue, "VALUES (CONVERT('TIMESTAMP', '13:20', 'TIME'))" => unnamed_0 = I64: (48000));
	assert_select!(glue, "VALUES (CONVERT('TIMESTAMP', '13:20:25', 'TIME'))" => unnamed_0 = I64: (48025));
	assert_select!(glue, "VALUES (CONVERT('TIMESTAMP', '2021-04-20', 22))" => unnamed_0 = I64: (1618876800));
	assert_select!(glue, "VALUES (CONVERT('TIMESTAMP', '2021-04-20', '%Y-%m-%d'))" => unnamed_0 = I64: (1618876800));

	assert_select!(glue,
		"VALUES (CONVERT('TEXT', 10000.921, 'MONEY'), CONVERT('TEXT', 10000.921, 'SEPARATED'))" => unnamed_0 = Str, unnamed_1 = Str:
		(String::from("$10,000.92"), String::from("10,000.92"))
	);
}
