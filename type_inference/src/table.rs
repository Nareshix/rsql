use std::collections::HashMap;
use std::ops::ControlFlow;

use sqlparser::ast::{ColumnOption, CreateTable, Statement, visit_relations};
use sqlparser::dialect::SQLiteDialect;
use sqlparser::parser::Parser;

use crate::expr::{BaseType, Type};

#[derive(Debug, Clone, PartialEq)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: Type,
    pub check_constraint: Option<String>,
}

fn convert_sqlite_to_rust_type(sql: String, nullable: bool) -> Type {
    let sql_upper = sql.to_uppercase();

    if sql_upper.contains("TEXT") {
        Type {
            base_type: BaseType::Text,
            nullable,
            contains_placeholder: false,
        }
        // also handles INTEGER
    } else if sql_upper.contains("INT") {
        Type {
            base_type: BaseType::Integer,
            nullable,
            contains_placeholder: false,
        }
    } else if sql_upper.contains("REAL")
        || sql_upper.contains("FLOAT")
        || sql_upper.contains("DOUBLE")
    {
        Type {
            base_type: BaseType::Real,
            nullable,
            contains_placeholder: false,
        }
    } else {
        Type {
            base_type: BaseType::Null,
            nullable,
            contains_placeholder: false,
        }
    }
    //TODO: BOOL
}

pub fn create_tables(sql: &str, tables: &mut HashMap<String, Vec<ColumnInfo>>) {
    let dialect = SQLiteDialect {};
    let ast = Parser::parse_sql(&dialect, sql).unwrap();

    for statement in ast {
        if let Statement::CreateTable(CreateTable { name, columns, .. }) = statement {
            let table_name = name.to_string();

            let table_columns = columns
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
                            // Unique and primary key are same in this context
                            ColumnOption::Unique {
                                is_primary: true, ..
                            } => {
                                // TODO
                            }
                            _ => {}
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
                .collect();

            tables.insert(table_name, table_columns);
        }
    }
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
