pub(crate) mod table;

macro_rules! all {
	($storage: ident) => {
		crate::functionality::statement::create::table::all!($storage);
	};
}
pub(crate) use all;
