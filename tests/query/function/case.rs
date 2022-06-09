use crate::util::*;
testcase!(test);
fn test(mut glue: multisql::Glue) {
	assert_select!(glue,
		"VALUES (CASE
				WHEN 1=0 THEN 1
				WHEN 1=1 THEN 2
				ELSE 3
			END)" => unnamed_0 = I64:
		(2)
	);
	assert_select!(glue,
		"VALUES (CASE
				WHEN 1=0 THEN 1
				WHEN 0=1 THEN 2
				ELSE 3
			END)" => unnamed_0 = I64:
		(3)
	);
	assert_select!(glue,
		"VALUES (CASE
				WHEN 1=1 THEN 1
				WHEN 0=1 THEN 2
				ELSE 3
			END)" => unnamed_0 = I64:
		(1)
	);
}
