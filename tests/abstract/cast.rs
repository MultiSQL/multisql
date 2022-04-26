crate::util_macros::testcase!(
	(|mut glue: multisql::Glue| {
		/* TODO: Redo this test */
		use super::Value::*, sqlparser::ast::DataType::*;
		
		macro_rules! cast {
			($input: expr => $data_type: expr, $expected: expr) => {
				let found = $input.cast_datatype(&$data_type).unwrap();

				match ($expected, found) {
					(Null, Null) => {}
					(expected, found) => {
						assert_eq!(expected, found);
					}
				}
			};
		}

		assert_ne!(Null, Null);
		assert_eq!(Bool(true), Bool(true));
		assert_eq!(I64(1), I64(1));
		assert_eq!(F64(6.11), F64(6.11));
		assert_eq!(Str("Glue".to_owned()), Str("Glue".to_owned()));

		// Same as
		cast!(Bool(true)            => Boolean      , Bool(true));
		cast!(Str("a".to_owned())   => Text         , Str("a".to_owned()));
		cast!(I64(1)                => Int(None)    , I64(1));
		cast!(F64(1.0)              => Float(None)  , F64(1.0));

		// Boolean
		cast!(Str("TRUE".to_owned())    => Boolean, Bool(true));
		cast!(Str("FALSE".to_owned())   => Boolean, Bool(false));
		cast!(I64(1)                    => Boolean, Bool(true));
		cast!(I64(0)                    => Boolean, Bool(false));
		cast!(F64(1.0)                  => Boolean, Bool(true));
		cast!(F64(0.0)                  => Boolean, Bool(false));
		cast!(Null                      => Boolean, Null);

		// Integer
		cast!(Bool(true)            => Int(None), I64(1));
		cast!(Bool(false)           => Int(None), I64(0));
		cast!(F64(1.1)              => Int(None), I64(1));
		cast!(Str("11".to_owned())  => Int(None), I64(11));
		cast!(Null                  => Int(None), Null);

		/*// Time // TODO
		cast!(Str("11:00".to_owned())  => Time, I64(11*60*60));
		cast!(Str("1:00PM".to_owned())  => Time, I64((12+1)*60*60));
		cast!(Str("23:35".to_owned())  => Time, I64((23*60*60) + 35*60));
		*/

		// Float
		cast!(Bool(true)            => Float(None), F64(1.0));
		cast!(Bool(false)           => Float(None), F64(0.0));
		cast!(I64(1)                => Float(None), F64(1.0));
		cast!(Str("11".to_owned())  => Float(None), F64(11.0));
		cast!(Null                  => Float(None), Null);

		// Text
		cast!(Bool(true)    => Text, Str("TRUE".to_owned()));
		cast!(Bool(false)   => Text, Str("FALSE".to_owned()));
		cast!(I64(11)       => Text, Str("11".to_owned()));
		cast!(F64(1.0)      => Text, Str("1.0".to_owned()));
		cast!(Null          => Text, Null);
	})
);
