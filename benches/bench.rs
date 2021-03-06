use {
	criterion::*,
	multisql::{Connection, Glue, Value},
	std::time::Duration,
};

fn setup_glue() -> Glue {
	let path = "data/sled_bench";

	match std::fs::remove_dir_all(&path) {
		Ok(()) => (),
		Err(e) => {
			println!("fs::remove_file {:?}", e);
		}
	}

	let storage = Connection::Sled(String::from("data/example_location/lib_example"))
		.try_into()
		.expect("Storage Creation Failed");

	Glue::new(String::from("main"), storage)
}

fn setup_a(glue: &mut Glue) {
	let rows: Vec<Vec<Value>> = (0..10_000).into_iter().map(|pk| vec![pk.into()]).collect();
	glue.execute(
		"
		CREATE TABLE A (
			pk INTEGER PRIMARY KEY
		)
	",
	)
	.unwrap();
	glue.execute(
		"
		CREATE INDEX primkey ON A (pk)
	",
	)
	.unwrap();
	glue.insert_vec(String::from("A"), vec![String::from("pk")], rows)
		.unwrap();
}

fn setup_b(glue: &mut Glue) {
	let rows: Vec<Vec<Value>> = (0..100_000)
		.into_iter()
		.map(|_row| vec![fastrand::i64(0..10_000).into(), fastrand::f64().into()])
		.collect();
	glue.execute(
		"
		CREATE TABLE B (
			pk INTEGER AUTO_INCREMENT PRIMARY KEY,
			fk INTEGER,
			val FLOAT
		)
	",
	)
	.unwrap();
	glue.execute(
		"
		CREATE INDEX primkey ON B (pk)
	",
	)
	.unwrap();
	glue.insert_vec(
		String::from("B"),
		vec![String::from("fk"), String::from("val")],
		rows,
	)
	.unwrap();
}

fn setup_c(glue: &mut Glue) {
	let rows: Vec<Vec<Value>> = (0..100_000)
		.into_iter()
		.map(|_row| vec![fastrand::i64(0..10_000).into(), fastrand::f64().into()])
		.collect();
	glue.execute(
		"
		CREATE TABLE C (
			pk INTEGER AUTO_INCREMENT PRIMARY KEY,
			fk INTEGER,
			val FLOAT
		)
	",
	)
	.unwrap();
	glue.insert_vec(
		String::from("C"),
		vec![String::from("fk"), String::from("val")],
		rows,
	)
	.unwrap();
}

fn setup() -> Glue {
	let mut glue = setup_glue();
	setup_a(&mut glue);
	setup_b(&mut glue);
	setup_c(&mut glue);
	glue
}

fn filter(table: &str) -> String {
	format!(
		"
		SELECT
			*
		FROM
			{}
		WHERE
			pk < 100
	",
		table
	)
}
fn find(table: &str) -> String {
	format!(
		"
		SELECT
			*
		FROM
			{}
		WHERE
			pk = 100
	",
		table
	)
}
fn sum_group(table: &str) -> String {
	format!(
		"
		SELECT
			SUM(val)
		FROM
			{}
		GROUP BY
			fk
	",
		table
	)
}
fn join(table: &str) -> String {
	format!(
		"
		SELECT
			SUM(val)
		FROM
			A
			INNER JOIN {table}
				ON {table}.fk = A.pk
		GROUP BY
			A.pk
	",
		table = table
	)
}

fn bench(criterion: &mut Criterion) {
	let mut glue = setup();

	let mut group = criterion.benchmark_group("filter");
	group.bench_function("a", |benchmarker| {
		benchmarker.iter(|| glue.execute(&filter("A")).unwrap());
	});
	group.bench_function("b", |benchmarker| {
		benchmarker.iter(|| glue.execute(&filter("B")).unwrap());
	});
	group.bench_function("c", |benchmarker| {
		benchmarker.iter(|| glue.execute(&filter("C")).unwrap());
	});
	group.finish();

	let mut group = criterion.benchmark_group("find");
	group.bench_function("a", |benchmarker| {
		benchmarker.iter(|| glue.execute(&find("A")).unwrap());
	});
	group.bench_function("b", |benchmarker| {
		benchmarker.iter(|| glue.execute(&find("B")).unwrap());
	});
	group.bench_function("c", |benchmarker| {
		benchmarker.iter(|| glue.execute(&find("C")).unwrap());
	});
	group.finish();

	let mut group = criterion.benchmark_group("sum_group");
	group
		.sampling_mode(SamplingMode::Flat)
		.measurement_time(Duration::from_secs(20));
	group.bench_function("b", |benchmarker| {
		benchmarker.iter(|| glue.execute(&sum_group("B")).unwrap());
	});
	group.bench_function("c", |benchmarker| {
		benchmarker.iter(|| glue.execute(&sum_group("C")).unwrap());
	});
	group.finish();

	let mut group = criterion.benchmark_group("join");
	group
		.sampling_mode(SamplingMode::Flat)
		.measurement_time(Duration::from_secs(30));
	group.bench_function("b", |benchmarker| {
		benchmarker.iter(|| glue.execute(&join("B")).unwrap());
	});
	group.bench_function("c", |benchmarker| {
		benchmarker.iter(|| glue.execute(&join("C")).unwrap());
	});
	group.finish();
}

criterion_group! {
	name = benches;
	config = Criterion::default().noise_threshold(0.05).sample_size(10).warm_up_time(Duration::from_secs(5)).measurement_time(Duration::from_secs(10));
	targets = bench
}
criterion_main!(benches);
