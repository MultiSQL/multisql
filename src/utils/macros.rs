macro_rules! warning {
	($expr: expr) => {
		println!("multisql Warning: {}", $expr);
	};
}
pub(crate) use warning;

macro_rules! try_option {
	($try: expr) => {
		match $try {
			Ok(success) => success,
			Err(error) => return Some(Err(error)),
		}
	};
}
pub(crate) use try_option;
