use crate::util::*;
testcase!(test);
fn test(mut glue: multisql::Glue) {
	assert_select!(
		glue,
		r#"
				VALUES (
					'Test',
					1
				), (
					'Test2',
					2
				), (
					'Test3',
					3
				), (
					'Test4',
					4
				)
				EXCEPT
				VALUES (
					'Test3',
					3
				)
			"# =>
			unnamed_0 = Str, unnamed_1 = I64:
			("Test", 1), ("Test2", 2), ("Test4", 4)
	);
}
