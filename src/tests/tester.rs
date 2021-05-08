use {
    crate::{
        executor::{execute, Payload},
        parse_sql::parse,
        result::Result,
        Storage,
    },
    async_trait::async_trait,
    std::{cell::RefCell, rc::Rc},
};

pub async fn run(cell: Rc<RefCell<Option<Storage>>>, sql: &str) -> Result<Payload> {
    println!("[Run] {}", sql);

    let mut storage = cell.replace(None).unwrap();

    let query = &parse(sql).unwrap()[0];
    let mut storage_inner = storage.take();
    let result = execute(vec![(String::new(), &mut *storage_inner)], query).await;
    storage.replace(storage_inner);

    cell.replace(Some(storage));

    result
}

/// If you want to make your custom storage and want to run integrate tests,
/// you should implement this `Tester` trait.
///
/// To see how to use it,
/// * [tests/sled_storage.rs](https://github.com/gluesql/gluesql/blob/main/tests/sled_storage.rs)
///
/// Actual test cases are in [/src/tests/](https://github.com/gluesql/gluesql/blob/main/src/tests/),
/// not in `/tests/`.
#[async_trait]
pub trait Tester {
    fn new(namespace: &str) -> Self;

    fn get_cell(&mut self) -> Rc<RefCell<Option<Storage>>>;
}

#[macro_export]
macro_rules! test_case {
    ($name: ident, $content: expr) => {
        pub async fn $name(mut tester: impl tests::Tester) {
            use std::rc::Rc;

            let cell = tester.get_cell();

            #[allow(unused_macros)]
            macro_rules! run {
                ($sql: expr) => {
                    tests::run(Rc::clone(&cell), $sql).await.unwrap()
                };
            }

            #[allow(unused_macros)]
            macro_rules! count {
                ($count: expr, $sql: expr) => {
                    match tests::run(Rc::clone(&cell), $sql).await.unwrap() {
                        Payload::Select { rows, .. } => {
                            rows.iter()
                                .enumerate()
                                .for_each(|(index, row)| println!("{} - {:?}", index, row));
                            assert_eq!($count, rows.len())
                        }
                        Payload::Delete(num) => assert_eq!($count, num),
                        Payload::Update(num) => assert_eq!($count, num),
                        _ => panic!("compare is only for Select, Delete and Update"),
                    };
                };
            }

            #[allow(unused_macros)]
            macro_rules! test {
                ($expected: expr, $sql: expr) => {
                    let found = tests::run(Rc::clone(&cell), $sql).await;

                    test($expected, found);
                };
            }

            #[allow(unused)]
            fn test(expected: Result<Payload>, found: Result<Payload>) {
                let (expected, found): (Payload, Payload) = match (expected, found) {
                    (Ok(a), Ok(b)) => (a, b),
                    (a, b) => {
                        assert_eq!(a, b);

                        return;
                    }
                };

                let (expected, found) = match (expected, found) {
                    (
                        Payload::Select {
                            labels: expected_labels,
                            rows: a,
                        },
                        Payload::Select {
                            labels: found_labels,
                            rows: b,
                        },
                    ) => {
                        // assert_eq!(expected_labels, found_labels); // TODO: Reenable

                        (a, b)
                    }
                    (a, b) => {
                        assert_eq!(a, b);

                        return;
                    }
                };

                assert_eq!(
                    expected.len(),
                    found.len(),
                    "\n[err: number of rows]\nexpected: {:?}\n   found: {:?}",
                    expected,
                    found
                );

                let rows = expected.into_iter().zip(found.into_iter()).enumerate();

                for (i, (expected, found)) in rows.into_iter() {
                    let Row(expected) = expected;
                    let Row(found) = found;

                    assert_eq!(
                        expected.len(),
                        found.len(),
                        "\n[err: size of row] row index: {}\nexpected: {:?}\n   found: {:?}",
                        i,
                        expected,
                        found
                    );

                    expected
                        .iter()
                        .zip(found.iter())
                        .for_each(|(expected_val, found_val)| {
                            if matches!((expected_val, found_val), (&Value::Null, &Value::Null)) {
                                return;
                            }

                            assert_eq!(
                                expected_val, found_val,
                                "\n[err: value] row index: {}\nexpected: {:?}\n   found: {:?}",
                                i, expected, found
                            );
                        });
                }
            }

            $content.await
        }
    };
}
