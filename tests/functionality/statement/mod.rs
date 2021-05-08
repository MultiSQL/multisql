pub(crate) mod create;
pub(crate) mod insert;
pub(crate) mod select;
pub(crate) mod set;

macro_rules! all {
	($storage: ident) => {
		crate::functionality::statement::set::all!($storage);
		crate::functionality::statement::select::all!($storage);
		crate::functionality::statement::insert::all!($storage);
		crate::functionality::statement::create::all!($storage);
	};
}
pub(crate) use all;
