crate::util_macros::testcase!(
	(|mut glue: multisql::Glue| {
		crate::util_macros::assert_success!(glue, "VALUES (NOW())");
		crate::util_macros::assert_select!(glue,
			"VALUES (CONVERT('TEXT', DATEFROMPARTS(2001,2,3), '%Y-%m-%d'))" => unnamed_0 = Str:
			(String::from("2001-02-03"))
		);
		crate::util_macros::assert_select!(glue,
			"VALUES (CONVERT('TEXT', 981158400, '%Y-%m-%d'))" => unnamed_0 = Str:
			(String::from("2001-02-03"))
		);
		crate::util_macros::assert_select!(glue,
			"VALUES (DATEFROMPARTS(2001,2,3))" => unnamed_0 = I64:
			(981158400)
		);

		crate::util_macros::assert_select!(glue,
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

		crate::util_macros::assert_select!(glue,
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
	})
);
