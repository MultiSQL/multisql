#![cfg(feature = "alter-table")]

use async_trait::async_trait;
use boolinator::Boolinator;
use std::iter::once;
use std::str;

use sqlparser::ast::{ColumnDef, Ident};

use super::{error::err_into, fetch_schema, SledStorage};
use crate::utils::Vector;
use crate::{schema::ColumnDefExt, AlterTable, AlterTableError, Result, Row, Schema, Value};

macro_rules! fetch_schema {
    ($self: expr, $tree: expr, $table_name: expr) => {{
        let (key, schema) = fetch_schema($tree, $table_name)?;
        let schema =
            schema.ok_or_else(|| AlterTableError::TableNotFound($table_name.to_string()))?;

        (key, schema)
    }};
}

#[async_trait(?Send)]
impl AlterTable for SledStorage {
    async fn rename_schema(&mut self, table_name: &str, new_table_name: &str) -> Result<()> {
        let (_, Schema { column_defs, .. }) = fetch_schema!(self, &self.tree, table_name);
        let schema = Schema {
            table_name: new_table_name.to_string(),
            column_defs,
        };

        let tree = &self.tree;

        // remove existing schema
        let key = format!("schema/{}", table_name);
        tree.remove(key).map_err(err_into)?;

        // insert new schema
        let value = bincode::serialize(&schema).map_err(err_into)?;
        let key = format!("schema/{}", new_table_name);
        let key = key.as_bytes();
        self.tree.insert(key, value).map_err(err_into)?;

        // replace data
        let prefix = format!("data/{}/", table_name);

        for item in tree.scan_prefix(prefix.as_bytes()) {
            let (key, value) = item.map_err(err_into)?;

            let new_key = str::from_utf8(key.as_ref()).map_err(err_into)?;
            let new_key = new_key.replace(table_name, new_table_name);
            tree.insert(new_key, value).map_err(err_into)?;

            tree.remove(key).map_err(err_into)?;
        }

        Ok(())
    }

    async fn rename_column(
        &mut self,
        table_name: &str,
        old_column_name: &str,
        new_column_name: &str,
    ) -> Result<()> {
        let (key, Schema { column_defs, .. }) = fetch_schema!(self, &self.tree, table_name);

        let i = column_defs
            .iter()
            .position(|column_def| column_def.name.value == old_column_name)
            .ok_or(AlterTableError::RenamingColumnNotFound);
        let i = i.map_err(err_into)?;

        let ColumnDef {
            name: Ident { quote_style, .. },
            data_type,
            collation,
            options,
        } = column_defs[i].clone();

        let column_def = ColumnDef {
            name: Ident {
                quote_style,
                value: new_column_name.to_string(),
            },
            data_type,
            collation,
            options,
        };
        let column_defs = Vector::from(column_defs).update(i, column_def).into();

        let schema = Schema {
            table_name: table_name.to_string(),
            column_defs,
        };
        let value = bincode::serialize(&schema).map_err(err_into)?;
        self.tree.insert(key, value).map_err(err_into)?;

        Ok(())
    }

    async fn add_column(&mut self, table_name: &str, column_def: &ColumnDef) -> Result<()> {
        let (
            key,
            Schema {
                table_name,
                column_defs,
            },
        ) = fetch_schema!(self, &self.tree, table_name);

        if column_defs
            .iter()
            .any(|ColumnDef { name, .. }| name.value == column_def.name.value)
        {
            let adding_column = column_def.name.value.to_string();

            return Err(AlterTableError::AddingColumnAlreadyExists(adding_column).into());
        }

        let ColumnDef { data_type, .. } = column_def;
        let nullable = column_def.is_nullable();
        let default = column_def.get_default();
        let value = match (default, nullable) {
            (Some(expr), _) => unimplemented!()/*try_self!(
                self,
                Value::from_expr(&data_type, nullable, expr)
            )*/,
            (None, true) => Value::Null,
            (None, false) => {
                return Err(
                    AlterTableError::DefaultValueRequired(column_def.to_string()).into(),
                );
            }
        };

        // migrate data
        let prefix = format!("data/{}/", table_name);

        for item in self.tree.scan_prefix(prefix.as_bytes()) {
            let (key, row) = item.map_err(err_into)?;
            let row: Row = bincode::deserialize(&row).map_err(err_into)?;
            let row = Row(row.0.into_iter().chain(once(value.clone())).collect());
            let row = bincode::serialize(&row).map_err(err_into)?;

            self.tree.insert(key, row).map_err(err_into)?;
        }

        // update schema
        let column_defs = column_defs
            .into_iter()
            .chain(once(column_def.clone()))
            .collect::<Vec<ColumnDef>>();

        let schema = Schema {
            table_name,
            column_defs,
        };
        let schema_value = bincode::serialize(&schema).map_err(err_into)?;
        self.tree.insert(key, schema_value).map_err(err_into)?;

        Ok(())
    }

    async fn drop_column(
        &mut self,
        table_name: &str,
        column_name: &str,
        if_exists: bool,
    ) -> Result<()> {
        let (
            key,
            Schema {
                table_name,
                column_defs,
            },
        ) = fetch_schema!(self, &self.tree, table_name);

        let index = column_defs
            .iter()
            .position(|ColumnDef { name, .. }| name.value == column_name);

        let index = match (index, if_exists) {
            (Some(index), _) => index,
            (None, true) => {
                return Ok(());
            }
            (None, false) => {
                return Err(
                    AlterTableError::DroppingColumnNotFound(column_name.to_string()).into(),
                );
            }
        };

        // migrate data
        let prefix = format!("data/{}/", table_name);

        for item in self.tree.scan_prefix(prefix.as_bytes()) {
            let (key, row) = item.map_err(err_into)?;
            let row: Row = bincode::deserialize(&row).map_err(err_into)?;
            let row = Row(row
                .0
                .into_iter()
                .enumerate()
                .filter_map(|(i, v)| (i != index).as_some(v))
                .collect());
            let row = bincode::serialize(&row).map_err(err_into)?;

            self.tree.insert(key, row).map_err(err_into)?;
        }

        // update schema
        let column_defs = column_defs
            .into_iter()
            .enumerate()
            .filter_map(|(i, v)| (i != index).as_some(v))
            .collect::<Vec<ColumnDef>>();

        let schema = Schema {
            table_name,
            column_defs,
        };
        let schema_value = bincode::serialize(&schema).map_err(err_into)?;
        self.tree.insert(key, schema_value).map_err(err_into)?;

        Ok(())
    }
}
