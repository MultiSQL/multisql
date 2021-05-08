pub(crate) mod set;

macro_rules! all {
	($storage: ident) => {
		crate::functionality::statement::set::all!($storage);
	};
}
pub(crate) use all;
