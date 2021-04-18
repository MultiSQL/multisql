use {
    super::{JoinError, JoinType},
    crate::{
        executor::{types::TableWithAlias, MetaRecipe},
        Result,
    },
    sqlparser::ast::{Join as AstJoin, JoinConstraint, JoinOperator, TableFactor},
};

#[derive(Debug, Clone)]
pub struct JoinManual {
    pub table: TableWithAlias,
    pub constraint: MetaRecipe,
    pub join_type: JoinType,
}

impl JoinManual {
    pub fn new(join: AstJoin) -> Result<Self> {
        let table = Self::table_identity(join.relation)?;
        let (join_type, constraint) = Self::convert_join(join.join_operator)?;
        // println!("(join/manual.rs) Constraint: {:?}", constraint);
        Ok(Self {
            table,
            join_type,
            constraint,
        })
    }
    pub fn new_implicit_join(table: TableFactor) -> Result<Self> {
        let table = Self::table_identity(table)?;
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
    pub fn table_identity(table: TableFactor) -> Result<TableWithAlias> {
        match table {
            TableFactor::Table { name, alias, .. } => {
                let name = name
                    .0
                    .get(0)
                    .ok_or(JoinError::UnimplementedNumberOfComponents)?
                    .value
                    .clone(); // We only support single component table names for now
                              // TODO
                let alias = alias.map(|alias| alias.name.value);

                Ok((alias, name))
            }
            _ => Err(JoinError::UnimplementedTableType.into()),
        }
    }
}
