macro_rules! all {
	($storage: ident) => {
		#[test]
		fn variable() {
			//use multisql::*;
			let mut glue = $storage();
			glue.execute("SET @variable = 1;").expect("SET variable");
		}
	};
}
pub(crate) use all;
