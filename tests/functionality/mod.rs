pub(crate) mod statement;

macro_rules! all {
	($storage: ident) => {
		crate::functionality::statement::all!($storage);
	};
}
pub(crate) use all;
