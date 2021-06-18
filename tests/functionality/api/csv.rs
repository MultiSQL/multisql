crate::util_macros::testcase!(
	(|mut glue: multisql::Glue| {
		crate::util_macros::make_basic_table!(glue);

		assert_eq!(
			glue.select_as_csv("SELECT * FROM basic"),
			Ok(String::from("a\n1\n"))
		);

		glue.execute("INSERT INTO basic VALUES (2),(3),(4),(5)")
			.unwrap();

		assert_eq!(
			glue.select_as_csv("SELECT * FROM basic ORDER BY a"),
			Ok(String::from("a\n1\n2\n3\n4\n5\n"))
		);

		glue.execute("ALTER TABLE basic ADD COLUMN empty FLOAT NULL")
			.unwrap();

		assert_eq!(
			glue.select_as_csv("SELECT * FROM basic ORDER BY a"),
			Ok(String::from(
				"a,empty\n1,NULL\n2,NULL\n3,NULL\n4,NULL\n5,NULL\n"
			))
		);
	})
);
