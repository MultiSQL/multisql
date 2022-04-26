# Benchmarks

## Results
[See results](https://htmlpreview.github.io/?https://github.com/KyGost/multisql/blob/main/benches/criterion/report/index.html)

## Tests
[See tests](./bench.rs)

## Hardware
2950X (AMD Ryzen 16 Core (32 Thread) CPU)
32GB 3000MHz (DDR4 RAM)
Running:
- Linux kernel 5.13.0-30-generic (64-bit)
- FerenOS 2021.10


## Simple overview
- Filtering 100,000 rows down to 100
	- 500 		μs indexed
	- 80,000 	μs unindexed
- Filtering 100,000 rows down to 1
	- 82,000 	μs indexed (index optimisations not yet implemented)
	- 86,000 	μs unindexed
- Grouping and summing 100,000 rows into 10,000 groups
	- 1,389,000 	μs indexed (index optimisations not yet implemented)
	- 1,421,000 	μs unindexed
- Joining 100,000 rows to 10,000 rows, grouping them into 10,000 groups and summing them
	- 588,000 	μs indexed (index optimisations not yet implemented)
	- 598,000 	μs unindexed
