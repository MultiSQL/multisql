use {
	crate::{Null, Value},
	enum_dispatch::enum_dispatch,
};

#[enum_dispatch(Value)]
pub trait BigEndian {
	fn to_be_bytes(&self) -> Vec<u8>;
}

const SEP: [u8; 1] = [0x00];
const NULL: [u8; 1] = [0x01];

impl BigEndian for Null {
	fn to_be_bytes(&self) -> Vec<u8> {
		[SEP, NULL].concat()
	}
}
impl BigEndian for bool {
	fn to_be_bytes(&self) -> Vec<u8> {
		[SEP, [if *self { 0x02 } else { 0x01 }]].concat()
	}
}
impl BigEndian for i64 {
	fn to_be_bytes(&self) -> Vec<u8> {
		[
			SEP.as_slice(),
			&[if self.is_positive() { 0x02 } else { 0x01 }],
			&self.to_be_bytes(),
		]
		.concat()
	}
}
impl BigEndian for u64 {
	fn to_be_bytes(&self) -> Vec<u8> {
		[SEP.as_slice(), &self.to_be_bytes()].concat()
	}
}
impl BigEndian for String {
	fn to_be_bytes(&self) -> Vec<u8> {
		[SEP.as_slice(), self.as_bytes()].concat()
	}
}

impl BigEndian for f64 {
	fn to_be_bytes(&self) -> Vec<u8> {
		unimplemented!()
	}
}
