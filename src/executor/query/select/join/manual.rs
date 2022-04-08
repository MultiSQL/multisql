use {
	super::{JoinError, JoinType},
	crate::{
		executor::{types::ComplexTableName, MetaRecipe},
		Context, Result,
	},
	sqlparser::ast::{Join as AstJoin, JoinConstraint, JoinOperator, TableFactor},
};

#[derive(Debug, Clone)]
pub struct JoinManual {
	pub table: ComplexTableName,
	pub constraint: MetaRecipe,
	pub join_type: JoinType,
}

impl JoinManual {
	pub fn new(join: AstJoin, context: &Context) -> Result<Self> {
		let table = join.relation.try_into()?;
		let (join_type, constraint) = Self::convert_join(join.join_operator)?;
		let constraint = constraint.simplify_by_context(context)?;
		Ok(Self {
			table,
			join_type,
			constraint,
		})
	}
	pub fn new_implicit_join(table: TableFactor) -> Result<Self> {
		let table = table.try_into()?;
		let (join_type, constraint) = (JoinType::CrossJoin, MetaRecipe::TRUE);
		Ok(Self {
			table,
			join_type,
			constraint,
		})
	}
	fn convert_join(from: JoinOperator) -> Result<(JoinType, MetaRecipe)> {
		let (join_type, constraint) = match from {
			JoinOperator::Inner(constraint) => (JoinType::Inner, Some(constraint)),
			JoinOperator::LeftOuter(constraint) => (JoinType::Left, Some(constraint)),
			JoinOperator::RightOuter(constraint) => (JoinType::Right, Some(constraint)),
			JoinOperator::FullOuter(constraint) => (JoinType::Full, Some(constraint)),
			JoinOperator::CrossJoin => (JoinType::CrossJoin, None),
			_ => return Err(JoinError::UnimplementedJoinType.into()),
		};
		let constraint = match constraint {
			Some(JoinConstraint::On(constraint)) => MetaRecipe::new(constraint)?,
			Some(JoinConstraint::None) | None => MetaRecipe::TRUE,
			_ => return Err(JoinError::UnimplementedJoinConstaint.into()),
		};
		Ok((join_type, constraint))
	}
}
