use std::cmp::Ordering;

use crate::{NullOrd, Value};

pub enum JoinType {
	Inner,
	Outer,
	Left,
	Right,
}

macro_rules! unwrap_or_break {
	($unwrap: expr) => {
		match $unwrap {
			Some(value) => value,
			None => {
				break;
			}
		}
	};
}

pub fn join_iters<Data>(join_type: JoinType, a: Vec<(Value, Data)>, b: Vec<(Value, Data)>) -> Vec<(Value, Data)>{
	// TODO: Only partition if not unique
	let mut a = partition_keyed_data(a)
		.into_iter()
		.peekable();
	let mut b = partition_keyed_data(b)
		.into_iter()
		.peekable();

	let mut results = vec![];
	// TODO: There's probably a better way to do this
	match join_type {
		Inner => loop {
			match unwrap_or_break!(a.peek())
				.0
				.null_cmp(&unwrap_or_break!(b.peek()).0).unwrap_or(Ordering::Equal)
			{
				Ordering::Equal => {
					results.push(a.next().unwrap());
					b.skip(1);
				}
				Ordering::Less => {
					a.skip(1);
				}
				Ordering::Greater => {
					b.skip(1);
				}
			}
		},
		Outer => loop {
			match unwrap_or_break!(a.peek())
				.0
				.null_cmp(&unwrap_or_break!(b.peek()).0).unwrap_or(Ordering::Equal)
			{
				Ordering::Less => {
					results
						.push(a.next().unwrap());
				}
				Ordering::Greater => {
					results
						.push(b.next().unwrap());
				}
				Ordering::Equal => {
					results.push(a.next().unwrap());
					b.skip(1);
				}
			}
			results.extend(a);
			results.extend(b);
		}
		Left => loop {
			match unwrap_or_break!(a.peek())
				.0
				.null_cmp(&unwrap_or_break!(b.peek()).0).unwrap_or(Ordering::Equal)
			{
				Ordering::Less => {
					results
						.push(a.next().unwrap());
				}
				Ordering::Equal => {
					results.push(a.next().unwrap());
					b.skip(1);
				}
				Ordering::Greater => {
					b.skip(1);
				}
			}
			results.extend(a);
		}
		Right => loop {
			match unwrap_or_break!(a.peek())
				.0
				.null_cmp(&unwrap_or_break!(b.peek()).0).unwrap_or(Ordering::Equal)
			{
				Ordering::Greater => {
					results
						.push(b.next().unwrap());
				}
				Ordering::Equal => {
					results.push(a.next().unwrap());
					b.skip(1);
				}
				Ordering::Less => {
					a.skip(1);
				}
			}
			results.extend(b);
		}
	}
	let results = results
		.into_iter()
		.reduce(
			|mut all, set| {
				all.extend(set);
				all
			},
		);
	results
}

pub fn partition_keyed_data<Data>(all: Vec<(Value, Data)>) -> Vec<(Value, Vec<Data>)>{
	all.into_iter()
		.fold(
			vec![],
			|mut partitions: Vec<(Value, Vec<Data>)>, (key, data)| {
				if let Some(last) = partitions.last_mut() {
					if last.0 == key {
						last.1.push(data);
					} else {
						partitions.push((key, vec![data]));
					}
					partitions
				} else {
					vec![(key, vec![data])]
				}
			},
		)
}
