use crate::Value;

pub trait BigEndian {
	fn to_be_bytes(&self) -> Vec<u8>;
}

const SEP: [u8; 1] = [0x00];
const NULL: [u8; 1] = [0x01];

impl BigEndian for Value {
	fn to_be_bytes(&self) -> Vec<u8> {
		use Value::*;
		match self {
			Null => [SEP, NULL].concat(),
			Bool(v) => [SEP, [if *v { 0x02 } else { 0x01 }]].concat(),
			I64(v) => [
				SEP.as_slice(),
				&[if v.is_positive() { 0x02 } else { 0x01 }],
				&v.to_be_bytes(),
			]
			.concat(),
			U64(v) => [SEP.as_slice(), &v.to_be_bytes()].concat(),
			Str(v) => [SEP.as_slice(), v.as_bytes()].concat(),
			_ => unimplemented!(),
		}
	}
}
