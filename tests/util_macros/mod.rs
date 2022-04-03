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
			rows: crate::util_macros::rows!(($($type),*) : $(($($value.into()),*)),*)
		}
	});
	($($label: tt = $type: ident),* : _) => ({ // Crappy but working way of testing single NULL
		multisql::Payload::Select {
			labels: vec![$( stringify!($label).to_owned().replace("\"", "")),+],
			rows: vec![multisql::Row(vec![multisql::Value::Null])]
		}
	});
}
pub(crate) use select;

macro_rules! execute {
	($storage: expr, $query: expr) => {{
		use fstrings::*;
		$storage
			.execute($query)
			.expect(&fstrings::f!("Query Failed: $query"));
	}};
}
pub(crate) use execute;

macro_rules! assert_select {
	($storage: expr, $query: expr => $($label: tt = $type: ident),* : $(($($value: expr),*)),*) => {{
		if let (
			Ok(multisql::Payload::Select { labels, mut rows }),
			multisql::Payload::Select { labels: expect_labels, rows: expect_rows }
		) = (
			$storage.execute($query),
			crate::util_macros::select!($($label = $type),* : $(($($value),*)),*)
		) {
			use fstrings::*;
			assert_eq!(labels, expect_labels);
			expect_rows.iter().for_each(|expect_row| {rows.remove(rows.iter().position(|row| expect_row == row).expect(&f!("\nRow missing: {expect_row:?}.\nQuery: {query}\nOther rows: {rows:?}", query=$query)));});
			rows.is_empty().then(||()).expect(&f!("Unexpected rows: {rows:?}\nQuery: {query}", query=$query));
		} else {
			let _result = $storage.execute($query);
			let _expect = crate::util_macros::select!($($label = $type),* : $(($($value),*)),*);
			panic!("SELECT Error\n\tQuery:\n\t{query}\n\tResult:\n\t{result:?}\n\tExpected:\t{expect:?}", query=$query, result=_result, expect=_expect);
		}
	}};
	($storage: expr, $query: expr => $($label: tt = $type: ident),* : $((_)),*) => {{ // Crappy but working way of testing single NULL
		if let (
			Ok(multisql::Payload::Select { labels, mut rows }),
			multisql::Payload::Select { labels: expect_labels, rows: expect_rows }
		) = (
			$storage.execute($query),
			crate::util_macros::select!($($label = $type),* : _)
		) {
			use fstrings::*;
			assert_eq!(labels, expect_labels);
			expect_rows.iter().for_each(|expect_row| {rows.remove(rows.iter().position(|_row| matches!(expect_row, _row)).expect(&f!("\nRow missing: {expect_row:?}.\nQuery: {query}\nOther rows: {rows:?}", query=$query)));});
			rows.is_empty().then(||()).expect(&f!("Unexpected rows: {rows:?}\nQuery: {query}", query=$query));
		} else {
			let _result = $storage.execute($query);
			let _expect = crate::util_macros::select!($($label = $type),* : _);
			panic!("SELECT Error\n\tQuery:\n\t{query}\n\tResult:\n\t{result:?}\n\tExpected:\t{expect:?}", query=$query, result=_result, expect=_expect);
		}
	}};
}
pub(crate) use assert_select;

macro_rules! assert_error {
	($storage: expr, $query: expr, $error: expr) => {{
		let _test: Result<(), _> = Err($error);
		matches!($storage.execute($query), _test)
			.then(|| ())
			.expect(&format!(
				"Unexexpected\n\tQuery:\n\t{query}\n\tExpected:\t{expect:?}",
				query = $query,
				expect = $error
			));
	}};
	($storage: expr, $query: expr) => {
		$storage.execute($query).expect_err(&format!(
			"Unexexpected Success\n\tQuery:\n\t{query}\n\tResult",
			query = $query
		));
	};
}
pub(crate) use assert_error;

macro_rules! assert_success {
	($storage: expr, $query: expr, $success: expr) => {{
		let _test: multisql::Result<_> = Ok($success);
		matches!($storage.execute($query), _test)
			.then(|| ())
			.expect(&format!(
				"Unexexpected\n\tQuery:\n\t{query}\n\tExpected:\t{expect:?}",
				query = $query,
				expect = $success
			));
	}};
	($storage: expr, $query: expr) => {
		$storage.execute($query).expect(&format!(
			"Unexexpected Error\n\tQuery:\n\t{query}\n\tResult",
			query = $query
		));
	};
}
pub(crate) use assert_success;

macro_rules! assert_result {
	($storage: expr, $query: expr, $success: expr) => {{
		let _test: multisql::Result<multisql::Payload> = $success;
		assert!(matches!($storage.execute($query), _test));
	}};
}
pub(crate) use assert_result;

macro_rules! assert_select_count {
	($storage: expr, $query: expr, $count: expr) => {{
		if let Ok(multisql::Payload::Select { rows, .. }) = $storage.execute($query) {
			assert_eq!(rows.len(), $count)
		} else {
			panic!(
				"Assert Select Count Failed\n\tQuery: {query}\n\tExpected: {count}",
				query = $query,
				count = $count
			)
		}
	}};
}
pub(crate) use assert_select_count;
