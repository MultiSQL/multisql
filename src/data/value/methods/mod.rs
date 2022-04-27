mod aggregate;
mod binary;
mod function;
mod timestamp;
mod unary;
mod utility;

pub use {
	binary::{BinaryOperation, BinaryOperations},
	unary::{UnaryOperation, UnaryOperations},
};

enum Operation {
	UnaryOperation(UnaryOperation),
	BinaryOperation(BinaryOperation),
}
