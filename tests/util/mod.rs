#![allow(unused_macros)]
#![allow(unused_imports)]

use multisql::Glue;

#[derive(Debug)]
pub(crate) struct Test {
	pub test: fn(Glue),
	pub name: &'static str,
}

macro_rules! testcase {
	($test: expr) => {
		inventory::submit!(Test {
			test: $test,
			name: module_path!(),
		});
	};
}
pub(crate) use testcase;
macro_rules! run {
	($test: expr, $storage: expr) => {
		use {
			indicatif::{ProgressBar, ProgressStyle},
			std::panic::catch_unwind,
		};
		let progress = ProgressBar::new_spinner().with_message($test.name);
		progress.enable_steady_tick(100);

		progress
			.set_style(ProgressStyle::default_spinner().template("[Running]\t {msg:50} {spinner}"));
		match catch_unwind(|| ($test.test)($storage($test.name))) {
			Ok(_) => {
				progress.set_style(
					ProgressStyle::default_spinner().template("[Passed]\t {msg:50.green}"),
				);
				progress.finish();
			}
			Err(err) => {
				progress.set_style(
					ProgressStyle::default_spinner().template("[Failed]\t {msg:50.red} {spinner}"),
				);
				println!("-\t Error:\t {:?}", err);
				progress.finish();
				break;
			}
		}
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
		let mut first = rows!(
			($($type),*):
			($($value),*)
		);
		first.append(
			&mut rows!(
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
			rows: rows!(($($type),*) : $(($($value.into()),*)),*)
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
		$storage
			.execute($query)
			.expect(&format!("Query Failed: {query}", query = $query));
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
			select!($($label = $type),* : $(($($value),*)),*)
		) {
			use fstrings::*;
			assert_eq!(labels, expect_labels);
			expect_rows.iter().for_each(|expect_row| {rows.remove(rows.iter().position(|row| expect_row == row).expect(&f!("\nRow missing: {expect_row:?}.\nQuery: {query}\nOther rows: {rows:?}", query=$query)));});
			rows.is_empty().then(||()).expect(&f!("Unexpected rows: {rows:?}\nQuery: {query}", query=$query));
		} else {
			let _result = $storage.execute($query);
			let _expect = select!($($label = $type),* : $(($($value),*)),*);
			panic!("SELECT Error\n\tQuery:\n\t{query}\n\tResult:\n\t{result:?}\n\tExpected:\t{expect:?}", query=$query, result=_result, expect=_expect);
		}
	}};
	($storage: expr, $query: expr => $($label: tt = $type: ident),* : $((_)),*) => {{ // Crappy but working way of testing single NULL
		if let (
			Ok(multisql::Payload::Select { labels, mut rows }),
			multisql::Payload::Select { labels: expect_labels, rows: expect_rows }
		) = (
			$storage.execute($query),
			select!($($label = $type),* : _)
		) {
			use fstrings::*;
			assert_eq!(labels, expect_labels);
			expect_rows.iter().for_each(|expect_row| {rows.remove(rows.iter().position(|_row| matches!(expect_row, _row)).expect(&f!("\nRow missing: {expect_row:?}.\nQuery: {query}\nOther rows: {rows:?}", query=$query)));});
			rows.is_empty().then(||()).expect(&f!("Unexpected rows: {rows:?}\nQuery: {query}", query=$query));
		} else {
			let _result = $storage.execute($query);
			let _expect = select!($($label = $type),* : _);
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
				"Unexpected\n\tQuery:\n\t{query}\n\tExpected:\t{expect:?}",
				query = $query,
				expect = _test
			));
	}};
	($storage: expr, $query: expr) => {
		$storage.execute($query).expect_err(&format!(
			"Unexpected Success\n\tQuery:\n\t{query}\n\tResult",
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
				"Unexpected\n\tQuery:\n\t{query}\n\tExpected:\t{expect:?}",
				query = $query,
				expect = $success
			));
	}};
	($storage: expr, $query: expr) => {
		$storage.execute($query).expect(&format!(
			"Unexpected Error\n\tQuery:\n\t{query}\n\tResult",
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
