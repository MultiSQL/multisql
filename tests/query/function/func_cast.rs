use crate::util::*;
testcase!(test);
fn test(mut glue: multisql::Glue) {
	execute!(glue, "CREATE TABLE Item (number TEXT)");
	execute!(glue, "INSERT INTO Item VALUES ('1')");

	assert_select!(glue, "
			SELECT CAST('true' AS BOOLEAN) AS cast FROM Item
			" => cast = Bool: (true)
	);
	assert_select!(glue, "
			SELECT CAST(1 AS BOOLEAN) AS cast FROM Item
			" => cast = Bool: (true)
	);
	/*assert_select!(glue, "
		SELECT CAST(NULL AS BOOLEAN) AS cast FROM Item
		" => cast = Null: ()
	);*/
	assert_select!(glue, "
			SELECT CAST('1' AS INTEGER) AS cast FROM Item
			" => cast = I64: (1)
	);
	assert_select!(glue, "
			SELECT CAST(1.1 AS INTEGER) AS cast FROM Item
			" => cast = I64: (1)
	);
	assert_select!(glue, "
			SELECT CAST(TRUE AS INTEGER) AS cast FROM Item
			" => cast = I64: (1)
	);
	/*assert_select!(glue, "
		SELECT CAST(NULL AS INTEGER) AS cast FROM Item
		" => cast = Null: ()
	);*/
	assert_select!(glue, "
			SELECT CAST('1.1' AS FLOAT) AS cast FROM Item
			" => cast = F64: (1.1)
	);
	assert_select!(glue, "
			SELECT CAST(1 AS FLOAT) AS cast FROM Item
			" => cast = F64: (1.0)
	);
	assert_select!(glue, "
			SELECT CAST(TRUE AS FLOAT) AS cast FROM Item
			" => cast = F64: (1.0)
	);
	/*assert_select!(glue, "
		SELECT CAST(NULL AS FLOAT) AS cast FROM Item
		" => cast = Null: ()
	);*/
	assert_select!(glue, "
			SELECT CAST(1 AS TEXT) AS cast FROM Item
			" => cast = Str: (String::from("1"))
	);
	assert_select!(glue, "
			SELECT CAST(1.1 AS TEXT) AS cast FROM Item
			" => cast = Str: (String::from("1.1"))
	);
	assert_select!(glue, "
			SELECT CAST(TRUE AS TEXT) AS cast FROM Item
			" => cast = Str: (String::from("true"))
	);
	/*assert_select!(glue, "
		SELECT CAST(NULL AS TEXT) AS cast FROM Item
		" => cast = Null: ()
	);*/
}
