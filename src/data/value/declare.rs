use {
	crate::{BigEndian, Cast},
	enum_dispatch::enum_dispatch,
	serde::{Deserialize, Serialize},
	std::fmt::Debug,
};

pub struct Null;

//#[enum_dispatch(Value)]
//pub trait Valued: BigEndian {}
//impl<T: Valued> BigEndian for T {}

macro_rules! value_types {
	( $($representation:ident: $type:ty),* ) => {
		//#[enum_dispatch(Value)]
		pub trait Valued: BigEndian $(+ Cast<$type>)* {}
		//$(impl Valued for $type {})*

		#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, Eq)]
		#[enum_dispatch(Valued)]
		pub enum Value {
			$($representation($type)),*
		}
	}
}

/*value_types!(
	Boolean:	bool,
	UInteger:	u64,
	Integer:	i64,
	Float:		f64,
	Text:			String,
);*/
value_types!(
	Bool: bool,
	U64: u64,
	I64: i64,
	F64: f64,
	Str: String,
	Null: Null
);
