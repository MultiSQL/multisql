#![allow(unused_macros)]
#![allow(unused_imports)]
macro_rules! make_all {
	($($path_part: ident)::*, [$($test: ident),*]) => {
		$(pub(crate) mod $test;)*
		macro_rules! all {
			($storage: ident, $name: tt) => {
				use crate::$($path_part)::*::{$($test),*};
				$($test::all!($storage, ($name, $test));)*
			};
		}
		pub(crate) use all;
	}
}
pub(crate) use make_all;

macro_rules! testcase {
	($test: expr) => {
		macro_rules! all {
			($storage: ident, $name: tt) => {
				crate::util_macros::make_test!($test, $storage, $name);
			};
		}
		pub(crate) use all;
	};
}
pub(crate) use testcase;

macro_rules! make_test {
	($test: expr, $storage: ident, ($($path_part: ident)::*)) => {
		concat_idents::concat_idents!(name = ""$(, $path_part,)"_"* {
			#[test]
			fn name() {
				$test($storage(concat!(""$(, stringify!($path_part),)"_"*)));
			}
		});
	};
	($test: expr, $storage: ident, (($($path_part: ident)::*), $($path_part_extension: ident)::+)) => {
		crate::util_macros::make_test!($test, $storage, ($($path_part)::*::$($path_part_extension)::*));
	};
	($test: expr, $storage: ident, (($more: tt, $extension: ident), $($path_part_extension: ident)::+)) => {
		crate::util_macros::make_test!($test, $storage, ($more, $extension::$($path_part_extension)::+));
	};
}
pub(crate) use make_test;

macro_rules! run {
	($storage: ident, $($path_part: ident)::*) => {
		crate::$($path_part)::*::all!($storage, ($($path_part)::*));
	};
}
pub(crate) use run;

macro_rules! make_basic_table {
	($glue: expr) => {
		$glue
			.execute(
				r#"
				CREATE TABLE basic (
					a INTEGER
				)
			"#,
			)
			.expect("CREATE TABLE basic");
		$glue
			.execute(
				r#"
				INSERT INTO basic (
					a
				) VALUES (
					1
				)
			"#,
			)
			.expect("INSERT basic");
	};
}
pub(crate) use make_basic_table;

macro_rules! rows {
	(($($type: ident),*) : ($($value: expr),*), $(($($value_todo: expr),*)),+) => {{
		let mut first = crate::util_macros::rows!(
			($($type),*):
			($($value),*)
		);
		first.append(
			&mut crate::util_macros::rows!(
				($($type),*):
				$(($($value_todo),*)),*
			)
		);
		first // Icky but works... is there a better way?
	}};
	(($($type: ident),*) : ($($value: expr),*)) => {
		vec![multisql::Row(vec![$(multisql::Value::$type($value)),*])]
	};
	(($($_type: ident),*) :) => {
		vec![]
	};
}
pub(crate) use rows;

macro_rules! select {
	($($label: tt = $type: ident),* : $(($($value: expr),*)),*) => ({
		multisql::Payload::Select {
			labels: vec![$( stringify!($label).to_owned().replace("\"", "")),+],
			rows: crate::util_macros::rows!(($($type),*) : $(($($value),*)),*)
		}
	});
}
pub(crate) use select;

macro_rules! assert_select {
	($storage: expr, $query: expr => $($label: tt = $type: ident),* : $(($($value: expr),*)),*) => {{
		if let (
			multisql::Payload::Select { labels, mut rows },
			multisql::Payload::Select { labels: expect_labels, rows: expect_rows }
		) = (
			$storage.execute($query).expect("SELECT Error"),
			crate::util_macros::select!($($label = $type),* : $(($($value),*)),*)
		) {
			use fstrings::*;
			assert_eq!(labels, expect_labels);
			expect_rows.iter().for_each(|expect_row| {rows.remove(rows.iter().position(|row| expect_row == row).expect(&f!("Row missing: {expect_row:?}")));});
			assert!(rows.is_empty(), "Unexpected rows: {rows:?}", rows = rows);
		} else {
			assert!(false);
		}
	}};
}
pub(crate) use assert_select;
