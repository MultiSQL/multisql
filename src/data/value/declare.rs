use {
	super::methods::{BinaryOperations, UnaryOperations},
	crate::{BigEndian, Cast, Error},
	enum_dispatch::enum_dispatch,
	serde::{Deserialize, Serialize},
	std::fmt::Debug,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct Null;

//#[enum_dispatch(Value)]
//pub trait Valued: BigEndian {}
//impl<T: Valued> BigEndian for T {}

macro_rules! value_types {
	( $($representation:ident: $type:ty),* ) => {
		#[enum_dispatch(Value)]
		pub trait Valued: BigEndian + BinaryOperations + UnaryOperations $(+ Cast<$type>)* {}
		//$(impl Valued for $type {})*

		#[enum_dispatch], Error
		#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
		pub enum Value {
			$($representation($type)),*
		}
	}
}

impl Value {
	pub const NULL: Self = Self::Null(Null);
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
