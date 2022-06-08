use crate::util::*;
testcase!(test);
fn test(mut glue: multisql::Glue) {
	use multisql::*;
	glue.execute("SET @variable = 1;").expect("SET variable");
	assert_eq!(
		glue.execute("VALUES (@variable)"),
		Ok(Payload::Select {
			labels: vec![String::from("unnamed_0")],
			rows: vec![Row(vec![Value::I64(1)])]
		})
	);
	make_basic_table!(glue);

	assert_eq!(
		glue.execute("SELECT a + @variable FROM basic"),
		Ok(Payload::Select {
			labels: vec![String::from("unnamed_0")],
			rows: vec![Row(vec![Value::I64(2)])]
		})
	);
	assert_eq!(
		glue.execute("SELECT a FROM basic WHERE @variable = 1"),
		Ok(Payload::Select {
			labels: vec![String::from("a")],
			rows: vec![Row(vec![Value::I64(1)])]
		})
	);
	assert_eq!(
		glue.execute("SELECT a FROM basic WHERE @variable = 0"),
		Ok(Payload::Select {
			labels: vec![String::from("a")],
			rows: vec![]
		})
	);
}
