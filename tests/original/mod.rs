pub(crate) mod basic;

macro_rules! all {
	($storage: ident) => {
		crate::original::basic::all!($storage);
	};
}
pub(crate) use all;
