use crate::util::*;
testcase!(test);
fn test(mut glue: multisql::Glue) {
	assert_success!(glue, "VALUES (NOW())");
	assert_select!(glue,
		"VALUES (CONVERT('TEXT', DATEFROMPARTS(2001,2,3), '%Y-%m-%d'))" => unnamed_0 = Str:
		(String::from("2001-02-03"))
	);
	assert_select!(glue,
		"VALUES (CONVERT('TEXT', 981158400, '%Y-%m-%d'))" => unnamed_0 = Str:
		(String::from("2001-02-03"))
	);
	assert_select!(glue,
		"VALUES (DATEFROMPARTS(2001,2,3))" => unnamed_0 = I64:
		(981158400)
	);
	assert_select!(glue,
		"VALUES (MONTH(981158400))" => unnamed_0 = I64:
		(2)
	);

	assert_select!(glue,
		"VALUES (CONVERT('TIMESTAMP', '2001-02-03 04:05:06', 'DATETIME'), DATEFROMPARTS(2001,2,3,4,5,6))" => unnamed_0 = I64, unnamed_1 = I64:
		(981173106, 981173106)
	);

	assert_select!(glue,
		"VALUES (YEAR(981173106), MONTH(981173106), DAY(981173106), HOUR(981173106), MINUTE(981173106), SECOND(981173106))" => unnamed_0 = I64, unnamed_1 = I64, unnamed_2 = I64, unnamed_3 = I64, unnamed_4 = I64, unnamed_5 = I64:
		(2001, 2, 3, 4, 5, 6)
	);

	assert_select!(glue,
		"VALUES (
				CONVERT('TEXT', 												981158400 , '%Y-%m-%d'),
				CONVERT('TEXT', DATEADD('DAY', 		10, 	981158400), '%Y-%m-%d'),
				CONVERT('TEXT', DATEADD('DAY', 		30, 	981158400), '%Y-%m-%d'),
				CONVERT('TEXT', DATEADD('DAY', 		365, 	981158400), '%Y-%m-%d'),
				CONVERT('TEXT', DATEADD('MONTH', 	1, 		981158400), '%Y-%m-%d'),
				CONVERT('TEXT', DATEADD('MONTH', 	13, 	981158400), '%Y-%m-%d'),
				CONVERT('TEXT', DATEADD('YEAR', 	1, 		981158400), '%Y-%m-%d')
			)" => unnamed_0 = Str, unnamed_1 = Str, unnamed_2 = Str, unnamed_3 = Str, unnamed_4 = Str, unnamed_5 = Str, unnamed_6 = Str: (
			String::from("2001-02-03"),
			String::from("2001-02-13"), // 10 day
			String::from("2001-03-05"), // 30 day
			String::from("2002-02-03"),	// 365 day
			String::from("2001-03-03"), // 1 month
			String::from("2002-03-03"), // 13 month
			String::from("2002-02-03") 	// 1 year
		)
	);

	assert_select!(glue,
		"VALUES (
				CONVERT('TEXT', 													981158400 , '%Y-%m-%d'),
				CONVERT('TEXT', DATEADD('DAY', 		-10, 		981158400), '%Y-%m-%d'),
				CONVERT('TEXT', DATEADD('DAY', 		-30, 		981158400), '%Y-%m-%d'),
				CONVERT('TEXT', DATEADD('DAY', 		-365, 	981158400), '%Y-%m-%d'),
				CONVERT('TEXT', DATEADD('MONTH', 	-1, 		981158400), '%Y-%m-%d'),
				CONVERT('TEXT', DATEADD('MONTH', 	-13, 		981158400), '%Y-%m-%d'),
				CONVERT('TEXT', DATEADD('YEAR', 	-1, 		981158400), '%Y-%m-%d')
			)" => unnamed_0 = Str, unnamed_1 = Str, unnamed_2 = Str, unnamed_3 = Str, unnamed_4 = Str, unnamed_5 = Str, unnamed_6 = Str: (
			String::from("2001-02-03"),
			String::from("2001-01-24"), // 10 day
			String::from("2001-01-04"), // 30 day
			String::from("2000-02-04"),	// 365 day
			String::from("2001-01-03"), // 1 month
			String::from("2000-01-03"), // 13 month
			String::from("2000-02-03") 	// 1 year
		)
	);
}
