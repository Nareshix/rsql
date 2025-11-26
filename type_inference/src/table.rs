use sqlparser::ast::{ColumnOption, CreateTable, Statement};
use sqlparser::dialect::SQLiteDialect;
use sqlparser::parser::Parser;



use crate::expr::Type;


// 1. The Structs
#[derive(Debug, Clone)]
#[allow(unused)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: Type,
    pub check_constraint: Option<String>,
}

#[derive(Debug)]
#[allow(unused)]
pub struct TableSchema {
    pub table_name: String,
    pub columns: Vec<ColumnInfo>,
}

fn convert_sqlite_to_rust_type(sql:String) -> Type{
    if sql == "TEXT"{
        Type::String
    } else if sql == "INTEGER" {
        Type::Int
    } else if sql == "REAL" {
        Type::Float
    } else {
        Type::Null
    }
    // TODO bool

}

/// Strictly only checks for Check COnstraints
pub fn create_table(sql: &str) -> Option<TableSchema> {
    let dialect = SQLiteDialect {};

    let ast = Parser::parse_sql(&dialect, sql).unwrap();

    if let Statement::CreateTable(CreateTable { name, columns, .. }) = &ast[0] {
        Some(TableSchema {
            table_name: name.to_string(),
            columns: columns
                .iter()
                .map(|col| {
                    let mut check_expr = None;

                    for option_def in &col.options {
                        if let ColumnOption::Check(expr) = &option_def.option {
                            // expr.to_string() turns the AST expression back into readable SQL
                            check_expr = Some(expr.to_string());
                        }
                    }

                    let data_type = convert_sqlite_to_rust_type(col.data_type.to_string());
                    ColumnInfo {
                        name: col.name.value.clone(),
                        data_type,
                        check_constraint: check_expr,
                    }
                })
                .collect(),
        })
    } else {
        None
    }
}
