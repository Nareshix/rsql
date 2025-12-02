use std::collections::HashMap;
use std::ops::ControlFlow;

use sqlparser::ast::{ColumnOption, CreateTable, Statement, visit_relations};
use sqlparser::dialect::SQLiteDialect;
use sqlparser::parser::Parser;

use crate::expr::{BaseType, Type};

// use crate::expr::Type;

// 1. The Structs
#[derive(Debug, Clone, PartialEq)]
#[allow(unused)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: Type,
    pub check_constraint: Option<String>,
}

#[derive(Debug)]
#[allow(unused)]
struct Table {
    pub table_name: String,
    pub columns: Vec<ColumnInfo>,
}

#[allow(unused)]
#[allow(unused)]
// 1. Add `nullable: bool` argument here
fn convert_sqlite_to_rust_type(sql: String, nullable: bool) -> Type {
    if sql == "TEXT" {
        Type {
            base_type: BaseType::Text,
            nullable,
            contains_placeholder: false
        }
    } else if sql == "INTEGER" {
        Type {
            base_type: BaseType::Integer,
            nullable,
            contains_placeholder: false
        }
    } else if sql == "REAL" {
        Type {
            base_type: BaseType::Real,
    nullable,
    contains_placeholder: false
        }
    } else {
        Type {
            base_type: BaseType::Null,
            nullable,
            contains_placeholder: false
        }
    }
    // TODO bool
}
#[allow(unused)]
/// parses the sql and creates an ast for table. then it  is inserted into the Hashmap.
pub fn create_tables(sql: &str, tables: &mut HashMap<String, Vec<ColumnInfo>>) {
    let dialect = SQLiteDialect {};
    let ast = Parser::parse_sql(&dialect, sql).unwrap();

    let schema = if let Statement::CreateTable(CreateTable { name, columns, .. }) = &ast[0] {
        Table {
            table_name: name.to_string(),
            columns: columns
                .iter()
                .map(|col| {
                    let mut check_expr = None;
                    let mut nullable = true;

                    for option_def in &col.options {
                        match &option_def.option {
                            ColumnOption::Check(expr) => {
                                check_expr = Some(expr.to_string());
                            }
                            ColumnOption::NotNull => {
                                nullable = false;
                            }
                            _ => {} // TODO
                        }
                    }

                    let data_type =
                        convert_sqlite_to_rust_type(col.data_type.to_string(), nullable);

                    ColumnInfo {
                        name: col.name.value.clone(),
                        data_type,
                        check_constraint: check_expr,
                    }
                })
                .collect(),
        }
    } else {
        panic!("Ensure that it is CREATE table"); //TODO
    };
    tables.insert(schema.table_name, schema.columns);
}

#[allow(unused)]
pub fn get_table_names(sql: &str) -> Vec<String> {
    let statements = Parser::parse_sql(&SQLiteDialect {}, sql).unwrap();
    let mut visited = vec![];
    let _ = visit_relations(&statements, |expr| {
        visited.push(expr.to_string());

        ControlFlow::<()>::Continue(())
    });

    visited

}
