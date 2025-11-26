use sqlparser::{ast::{BinaryOperator, Expr, SelectItem, SetExpr, Statement, Value, Visit, Visitor}, dialect::SQLiteDialect, parser::Parser};
use std::{collections::HashMap, ops::ControlFlow};

use crate::{expr::infer_type, table::create_table};
mod table;
mod expr;
fn main() {
    let sql = "CREATE TABLE users (id INTEGER, name TEXT, wow REAL);";
    let schema = create_table(sql).unwrap();

    let mut tables = HashMap::new();
    tables.insert(schema.table_name.clone(), schema.columns.clone());

    
    // Changed SQL to explicit columns because '*' is a Wildcard, not an Expr
    let sql = "SELECT id+1.0, name, wow FROM users";

    let ast = Parser::parse_sql(&SQLiteDialect {}, sql).unwrap();

    // We take the first statement
    match &ast[0] {
        // Check if it is a Query (SELECT)
        Statement::Query(query) => {
            // Check the body of the query (Standard SELECT)
            if let SetExpr::Select(select) = &*query.body {
                // Iterate over the columns requested (projection)
                for item in &select.projection {
                    match item {
                        // Case: SELECT id ...
                        SelectItem::UnnamedExpr(expr) => {
                            let inferred = infer_type(expr, &schema);
                            println!("Expression: {:?}, Type: {:?}", expr, inferred);
                        }
                        // Case: SELECT id AS user_id ...
                        SelectItem::ExprWithAlias { expr, alias } => {
                            let inferred = infer_type(expr, &schema);
                            println!("Alias: {}, Type: {:?}", alias, inferred);
                        }
                        // Case: SELECT * ...
                        SelectItem::Wildcard(_) => {
                            println!("Wildcard (*) - logic needs to expand schema columns");
                        }
                        _ => println!("Other select item type"),
                    }
                }
            }
        }
        _ => println!("Not a select statement"),
    }
}
