use {
    super::{
        super::{Recipe, Subquery},
        recipe, table_identity, ObjectName, TableIdentity,
    },
    crate::{Result, Table},
    sqlparser::ast::{Join as AstJoin, JoinConstraint, JoinOperator as JoinOperatorAst},
};

pub type Join = (TableIdentity, (JoinOperator, Recipe, Vec<ObjectName>));
pub enum JoinOperator {
    Inner,
    Left,
    Right,
    Full,
}

pub fn map_join(join: AstJoin) -> Result<Join> {
    Ok((
        table_identity(Table::new(&join.relation)?),
        convert_join(join.join_operator)?,
    ))
}

pub fn map_subquery_to_join(subquery: Subquery) -> Join {
    (
        (String::new() /* Alias */, subquery.table),
        (JoinOperator::Left, subquery.constraint, vec![]),
    )
}

pub fn convert_join(from: JoinOperatorAst) -> Result<(JoinOperator, Recipe, Vec<ObjectName>)> {
    let mut columns = vec![];
    let values = match from {
        JoinOperatorAst::Inner(JoinConstraint::On(constraint)) => (JoinOperator::Inner, constraint),
        _ => unimplemented!(),
    };
    Ok((values.0, recipe(values.1, &mut columns)?, columns))
}
