use crate::util::*;
testcase!(test);
fn test(_glue: multisql::Glue) {
	use multisql::{Cast, Value::*};
	let bool_equivs = vec![
		(Bool(false), Str(String::from("false"))),
		(Bool(false), I64(0)),
		(Bool(false), F64(0.0)),
		(Bool(true), Str(String::from("true"))),
		(Bool(true), I64(1)),
		(Bool(true), F64(1.0)),
	];
	bool_equivs.iter().for_each(|(a, b)| {
		let a_inner: bool = a.clone().cast().unwrap();
		let b_cast: bool = b.clone().cast().unwrap();
		assert_eq!(a_inner, b_cast);
	});

	let int_equivs = vec![
		(I64(0), Bool(false)),
		(I64(0), Str(String::from("0"))),
		(I64(0), F64(0.0)),
		(I64(1), Bool(true)),
		(I64(1), Str(String::from("1"))),
		(I64(1), F64(1.0)),
		(I64(999), Str(String::from("999"))),
		(I64(999), F64(999.0)),
	];
	int_equivs.iter().for_each(|(a, b)| {
		let a_inner: i64 = a.clone().cast().unwrap();
		let b_cast: i64 = b.clone().cast().unwrap();
		assert_eq!(a_inner, b_cast);
	});

	let float_equivs = vec![
		(F64(0.0), Bool(false)),
		(F64(0.0), Str(String::from("0.0"))),
		(F64(0.0), I64(0)),
		(F64(1.0), Bool(true)),
		(F64(1.0), Str(String::from("1.0"))),
		(F64(1.0), I64(1)),
		(F64(999.99), Str(String::from("999.99"))),
		(F64(999.0), I64(999)),
	];
	float_equivs.iter().for_each(|(a, b)| {
		let a_inner: f64 = a.clone().cast().unwrap();
		let b_cast: f64 = b.clone().cast().unwrap();
		assert_eq!(a_inner, b_cast);
	});

	let str_equivs = vec![
		(Str(String::from("false")), Bool(false)),
		(Str(String::from("0")), I64(0)),
		(Str(String::from("0.0")), F64(0.0)),
		(Str(String::from("true")), Bool(true)),
		(Str(String::from("1")), I64(1)),
		(Str(String::from("1.0")), F64(1.0)),
	];
	str_equivs.iter().for_each(|(a, b)| {
		let a_inner: String = a.clone().cast().unwrap();
		let b_cast: String = b.clone().cast().unwrap();
		assert_eq!(a_inner, b_cast);
	});

	// TODO: Error cases
}
