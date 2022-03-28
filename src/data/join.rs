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

pub fn join_iters(join_type: JoinType, a: Vec<Value>, b: Vec<Value>) -> Vec<Value> {
	let mut a = a.into_iter().peekable();
	let mut b = b.into_iter().peekable();
	let mut results = vec![];
	// TODO: There's probably a better way to do this
	match join_type {
		JoinType::Inner => loop {
			match unwrap_or_break!(a.peek())
				.null_cmp(unwrap_or_break!(&b.peek()))
				.unwrap_or(Ordering::Equal)
			{
				Ordering::Equal => {
					results.push(a.next().unwrap());
					b.next();
				}
				Ordering::Less => {
					a.next();
				}
				Ordering::Greater => {
					b.next();
				}
			}
		},
		JoinType::Outer => {
			loop {
				match unwrap_or_break!(a.peek())
					.null_cmp(unwrap_or_break!(&b.peek()))
					.unwrap_or(Ordering::Equal)
				{
					Ordering::Less => {
						results.push(a.next().unwrap());
					}
					Ordering::Greater => {
						results.push(b.next().unwrap());
					}
					Ordering::Equal => {
						results.push(a.next().unwrap());
						b.next();
					}
				}
			}
			results.extend(a);
			results.extend(b);
		}
		JoinType::Left => {
			loop {
				match unwrap_or_break!(a.peek())
					.null_cmp(unwrap_or_break!(b.peek()))
					.unwrap_or(Ordering::Equal)
				{
					Ordering::Less => {
						results.push(a.next().unwrap());
					}
					Ordering::Equal => {
						results.push(a.next().unwrap());
						b.next();
					}
					Ordering::Greater => {
						b.next();
					}
				}
			}
			results.extend(a);
		}
		JoinType::Right => {
			loop {
				match unwrap_or_break!(a.peek())
					.null_cmp(unwrap_or_break!(&b.peek()))
					.unwrap_or(Ordering::Equal)
				{
					Ordering::Greater => {
						results.push(b.next().unwrap());
					}
					Ordering::Equal => {
						results.push(a.next().unwrap());
						b.next();
					}
					Ordering::Less => {
						a.next();
					}
				}
			}
			results.extend(b);
		}
	}
	results
}
