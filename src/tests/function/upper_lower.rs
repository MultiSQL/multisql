use crate::*;

test_case!(upper_lower, async move {
    use executor::recipe::method::Function::{Lower, Upper};
    use Value::{Null, Str};

    let test_cases = vec![
        (
            "CREATE TABLE Item (name TEXT, opt_name TEXT NULL)",
            Ok(Payload::Create),
        ),
        (
            r#"INSERT INTO Item VALUES ("abcd", "efgi"), ("Abcd", NULL), ("ABCD", "EfGi")"#,
            Ok(Payload::Insert(3)),
        ),
        (
            r#"SELECT name FROM Item WHERE LOWER(name) = "abcd";"#,
            Ok(select!(
                name Str;
                "abcd".to_owned();
                "Abcd".to_owned();
                "ABCD".to_owned()
            )),
        ),
        (
            "SELECT LOWER(name), UPPER(name) FROM Item;",
            Ok(select!(
                "LOWER(name)"      | "UPPER(name)"
                Str                | Str;
                "abcd".to_owned()    "ABCD".to_owned();
                "abcd".to_owned()    "ABCD".to_owned();
                "abcd".to_owned()    "ABCD".to_owned()
            )),
        ),
        (
            r#"
            SELECT
                LOWER("Abcd") as lower,
                UPPER("abCd") as upper
            FROM Item LIMIT 1;
            "#,
            Ok(select!(
                lower             | upper
                Str               | Str;
                "abcd".to_owned()   "ABCD".to_owned()
            )),
        ),
        (
            "SELECT LOWER(opt_name), UPPER(opt_name) FROM Item;",
            Ok(select_with_null!(
                "LOWER(opt_name)"      | "UPPER(opt_name)";
                Str("efgi".to_owned())   Str("EFGI".to_owned());
                Null                     Null;
                Str("efgi".to_owned())   Str("EFGI".to_owned())
            )),
        ),
        (
            "SELECT LOWER() FROM Item",
            Err(CalculationError::WrongNumberOfArguments {
                function: Lower,
                expected: 1,
                found: 0,
            }
            .into()),
        ),
        (
            "SELECT LOWER(1) FROM Item",
            Err(CalculationError::FunctionRequiresDataType {
                function: Lower,
                expected: Value::Str(String::from("")),
                found: Value::I64(1),
            }
            .into()),
        ),
        (
            "SELECT WHATEVER(1) FROM Item",
            Err(RecipeError::UnimplementedMethod("WHATEVER".to_owned()).into()),
        ),
    ];

    for (sql, expected) in test_cases.into_iter() {
        test!(expected, sql);
    }
});
