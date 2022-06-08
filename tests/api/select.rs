use {crate::util::*, multisql::*};
testcase!(test);
fn test(mut glue: Glue) {
	make_basic_table!(glue);

	assert_eq!(
		SELECT! {glue, a FROM main.basic},
		Ok(Payload::Select {
			labels: vec!["a".into()],
			rows: vec![Row(vec![Value::I64(1)])]
		})
	);

	/*
	assert_eq!(
		SELECT! {glue, a FROM basic},
		Ok(Payload::Select {
			labels: vec!["a".into()],
			rows: vec![Row(vec![Value::I64(1)])]
		})
	);

	assert_eq!(
		SELECT! {glue, * FROM basic},
		Ok(Payload::Select {
			labels: vec!["a".into()],
			rows: vec![Row(vec![Value::I64(1)])]
		})
	);

	assert_eq!(
		SELECT! {glue, a FROM basic WHERE a = 1},
		Ok(Payload::Select {
			labels: vec!["a".into()],
			rows: vec![Row(vec![Value::I64(1)])]
		})
	);

	assert_eq!(
		SELECT! {glue, a FROM basic WHERE a = 2},
		Ok(Payload::Select {
			labels: vec!["a".into()],
			rows: vec![]
		})
	);*/
}
