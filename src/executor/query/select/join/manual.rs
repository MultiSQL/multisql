use {
    super::{JoinError, JoinType},
    crate::{
        executor::{types::TableWithAlias, MetaRecipe, Recipe},
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
        let values = match from {
            JoinOperator::Inner(JoinConstraint::On(constraint)) => (JoinType::Inner, constraint),
            JoinOperator::LeftOuter(JoinConstraint::On(constraint)) => (JoinType::Left, constraint),
            JoinOperator::RightOuter(JoinConstraint::On(constraint)) => {
                (JoinType::Right, constraint)
            }
            JoinOperator::FullOuter(JoinConstraint::On(constraint)) => (JoinType::Full, constraint),
            JoinOperator::CrossJoin => return Ok((JoinType::CrossJoin, MetaRecipe::TRUE)),
            _ => return Err(JoinError::UnimplementedJoinType.into()),
        };
        Ok((values.0, MetaRecipe::new(values.1)?))
    }
    pub fn table_identity(table: TableFactor) -> Result<TableWithAlias> {
        match table {
            TableFactor::Table { name, alias, .. } => {
                let name = name.0.get(0).ok_or(JoinError::Unreachable)?.value.clone(); // We only support single component table names for now
                                                                                       // TODO
                let alias = alias.map(|alias| alias.name.value);

                Ok((alias, name))
            }
            _ => Err(JoinError::UnimplementedTableType.into()),
        }
    }
}
