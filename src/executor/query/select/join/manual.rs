use {
    super::{JoinError, JoinType},
    crate::{
        executor::{types::ComplexTableName, MetaRecipe},
        Result,
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
    pub fn table_identity(table: TableFactor) -> Result<ComplexTableName> {
        match table {
            TableFactor::Table { name, alias, .. } => {
                let name_parts = name.0.len();
                if name_parts > 2 || name_parts < 1 {
                    return Err(JoinError::UnimplementedNumberOfComponents.into());
                }
                let database = if name_parts == 2 {
                    name.0.get(0).unwrap().value.clone()
                } else {
                    String::new()
                };
                let name = name.0.last().unwrap().value.clone();
                let alias = alias.map(|alias| alias.name.value);
                Ok(ComplexTableName {
                    database,
                    name,
                    alias,
                })
            }
            _ => Err(JoinError::UnimplementedTableType.into()),
        }
    }
}
