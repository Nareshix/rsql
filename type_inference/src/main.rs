// use crate::select_patterns::get_types_from_select;
// use crate::table::create_tables;
// use std::collections::HashMap;

mod expr;
mod select_patterns;
mod table;

// fn main() {
//     let mut all_tables = HashMap::new();

//     let sql = "CREATE TABLE users (id INTEGER, name TEXT, wow REAL)";
//     create_tables(sql, &mut all_tables);

//     let sql = "CREATE TABLE mom (id INTEGER, name TEXT NOT NULL, wow TEXT)";
//     create_tables(sql, &mut all_tables);

//     let sql = "SELECT * FROM mom";
//     let types = get_types_from_select(sql, &all_tables);
//     println!("{:?}", types);

//     let sql = "SELECT id > 'asd' FROM mom";
//     let types = get_types_from_select(sql, &all_tables);
//     println!("{:?}", types);

//     let sql = "SELECT not NULL";
//     let types = get_types_from_select(sql, &all_tables);
//     println!("{:?}", types);

// }

use crate::{
    expr::evaluate_expr_type,
    table::{create_tables, get_table_names},
};
use sqlparser::{
    ast::{Expr, Value, ValueWithSpan, visit_expressions},
    dialect::SQLiteDialect,
    parser::Parser,
};
use std::{collections::HashMap, ops::ControlFlow};

fn main() {
    let mut all_tables = HashMap::new();

    let sql = "CREATE TABLE users (id INTEGER, name TEXT, wow REAL)";
    create_tables(sql, &mut all_tables);

    let sql = "CREATE TABLE mom (id INTEGER NOT NULL, name TEXT NOT NULL, wow TEXT)";
    create_tables(sql, &mut all_tables);

    let sql = "Select 1 from users where name = ? AND wow + 1 > ? OR id in (?,?,?)";
    let statements = Parser::parse_sql(&SQLiteDialect {}, sql).unwrap();

    let table_names_from_select = get_table_names(sql);
    let _ = visit_expressions(&statements, |expr| {
        match expr {
            Expr::BinaryOp { left, right, .. } => {
                if let Expr::Value(ValueWithSpan { value, .. }) = &**right
                    && let Value::Placeholder(_) = value
                {
                    let x =
                        evaluate_expr_type(left, &table_names_from_select, &all_tables).unwrap();
                    println!("{:?}", x);
                }
            }
            Expr::InList { expr, list, .. } => {
                // assume that all the InList contains ?
                if let Expr::Value(ValueWithSpan { value, .. }) = &list[0]
                    && let Value::Placeholder(_) = value
                {
                    let x =
                        evaluate_expr_type(expr, &table_names_from_select, &all_tables).unwrap();
                    println!("{:?}", x);
                }
            }

            Expr::Between {
                expr, low, high, ..
            } => {
                if let Expr::Value(ValueWithSpan { value, .. }) = &**low
                    && let Value::Placeholder(_) = value
                {
                    let x =
                        evaluate_expr_type(expr, &table_names_from_select, &all_tables).unwrap();
                    println!("{:?}", x);
                } else if let Expr::Value(ValueWithSpan { value, .. }) = &**high
                    && let Value::Placeholder(_) = value
                {
                    let x =
                        evaluate_expr_type(expr, &table_names_from_select, &all_tables).unwrap();
                    println!("{:?}", x);
                }
            }
            _ => {}
        }

        ControlFlow::<()>::Continue(())
    });
}
