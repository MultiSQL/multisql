use crate::util::*;
testcase!(test);
fn test(mut glue: multisql::Glue) {
	assert_success!(
		glue,
		"CREATE TABLE Item (name TEXT, opt_name TEXT NULL)",
		multisql::Payload::Create
	);
	assert_success!(
		glue,
		"INSERT INTO Item VALUES ('abcd', 'efgi'), ('Abcd', NULL), ('ABCD', 'EfGi')",
		multisql::Payload::Insert(3)
	);
	assert_select!(glue,
		"SELECT name FROM Item WHERE LOWER(name) = 'abcd'" => name = Str:
		(String::from("abcd")),
		(String::from("Abcd")),
		(String::from("ABCD"))
	);
	assert_select!(glue,
		"SELECT LOWER(name) AS lower, UPPER(name) as upper FROM Item;" => lower = Str, upper = Str:
		((String::from("abcd")), (String::from("ABCD"))),
		((String::from("abcd")), (String::from("ABCD"))),
		((String::from("abcd")), (String::from("ABCD")))
	);
	assert_select!(glue,
		"VALUES (LOWER('Abcd'), UPPER('abCd'))" => unnamed_0 = Str, unnamed_1 = Str:
		((String::from("abcd")), (String::from("ABCD")))
	);
	/* TODO: Null test
	assert_select!(glue,
		"SELECT LOWER(opt_name) AS lower, UPPER(opt_name) as upper FROM Item;" => lower = Str, upper = Str:
		((String::from("efgi")), (String::from("EFGI"))),
		(), (),
		((String::from("efgi")), (String::from("EFGI")))
	);*/
	assert_error!(
		glue,
		"SELECT LOWER() FROM Item",
		multisql::ValueError::NumberOfFunctionParamsNotMatching {
			expected: 1,
			found: 0,
		}
	);
	assert_error!(
		glue,
		"SELECT LOWER(1) FROM Item",
		multisql::ValueError::CannotConvert(multisql::Value::I64(1), "TEXT")
	);
	assert_error!(
		glue,
		"SELECT WHATEVER(1) FROM Item",
		multisql::RecipeError::UnimplementedMethod(String::from("WHATEVER"))
	);
}
