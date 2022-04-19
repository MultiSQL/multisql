use {
	serde::{Deserialize, Serialize},
	std::fmt::Debug,
	enum_dispatch::enum_dispatch,
};

#[enum_dispatch]
pub trait Valued {

}

macro_rules! value_types {
	( $($representation:ident: $type:ty),* ) => {
		$(impl Valued for $type {})*

		#[derive(Debug, Clone, Serialize, Deserialize)]
		#[enum_dispatch(Valued)]
		pub enum Value {
			Null,
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
	Bool:	bool,
	U64:	u64,
	I64:	i64,
	F64:	f64,
	Str:	String
);
