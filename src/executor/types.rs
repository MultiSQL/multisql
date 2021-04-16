use {
    crate::{executor::Recipe, Value},
    std::fmt::Debug,
};

pub type Table = String;
pub type TableWithAlias = (Alias, Table);
pub type Alias = Option<String>;
pub type Label = String;
pub type Row = Vec<Value>;
pub type LabelsAndRows = (Vec<Label>, Vec<Row>);
pub type ObjectName = Vec<String>;

#[derive(Debug, Clone)]
pub struct ComplexColumnName {
    pub table: TableWithAlias,
    pub name: String,
}

impl PartialEq<ObjectName> for ComplexColumnName {
    fn eq(&self, other: &ObjectName) -> bool {
        let mut other = other.clone();
        other.reverse();
        let names_eq = other
            .get(0)
            .map(|column| column == &self.name)
            .unwrap_or(false);
        let tables_eq = other
            .get(1)
            .map(|table| {
                table == &self.table.1
                    || self
                        .table
                        .0
                        .as_ref()
                        .map(|alias| table == alias)
                        .unwrap_or(false)
            })
            .unwrap_or(true);
        names_eq && tables_eq
    }
}
