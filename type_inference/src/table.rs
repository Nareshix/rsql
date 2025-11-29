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
        }
    } else if sql == "INTEGER" {
        Type {
            base_type: BaseType::Integer,
            nullable,
        }
    } else if sql == "REAL" {
        Type {
            base_type: BaseType::Real,
            nullable,
        }
    } else {
        Type {
            base_type: BaseType::Null,
            nullable,
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
/// does not support ctes and subqueries
pub fn get_table_names(sql: &str) -> Vec<String> {
    let statements = Parser::parse_sql(&SQLiteDialect {}, sql).unwrap();
    let mut visited = vec![];
    let _ = visit_relations(&statements, |expr| {
        visited.push(expr.to_string());

        ControlFlow::<()>::Continue(())
    });

    visited

}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     // Helper to parse SQL quickly
//     fn extract(sql: &str) -> Vec<String> {
//         let dialect = SQLiteDialect {};
//         let ast = Parser::parse_sql(&dialect, sql).unwrap();
//         match &ast[0] {
//             sqlparser::ast::Statement::Query(q) => get_table_names(q),
//             _ => vec![],
//         }
//     }

//     #[test]
//     fn test_basic_tables() {
//         let tables = extract("SELECT * FROM users");
//         assert_eq!(tables, vec!["users"]);
//     }

//     #[test]
//     fn test_comma_separated() {
//         // Handles "FROM A, B"
//         let tables = extract("SELECT * FROM users, accounts");
//         assert_eq!(tables, vec!["users", "accounts"]);
//     }

//     #[test]
//     fn test_standard_joins() {
//         // Handles "FROM A JOIN B JOIN C"
//         let tables = extract(
//             "
//             SELECT *
//             FROM users
//             JOIN orders ON users.id = orders.uid
//             LEFT JOIN items ON orders.id = items.oid
//         ",
//         );
//         assert_eq!(tables, vec!["users", "orders", "items"]);
//     }

//     #[test]
//     fn test_ignore_ctes() {
//         // Should NOT find "cte_table"
//         // Should find "real_table"
//         let sql = "
//             WITH cte_table AS (SELECT * FROM hidden)
//             SELECT * FROM real_table
//         ";
//         let tables = extract(sql);
//         assert_eq!(tables, vec!["real_table"]);
//     }

//     #[test]
//     fn test_ignore_subqueries() {
//         // Should NOT find "hidden" inside the subquery
//         // Should find "users"
//         let sql = "
//             SELECT *
//             FROM users
//             JOIN (SELECT * FROM hidden) AS sub ON sub.id = users.id
//         ";
//         let tables = extract(sql);
//         assert_eq!(tables, vec!["users"]);
//     }

//     #[test]
//     fn test_nested_joins() {
//         // Handles parentheses: FROM (A JOIN B)
//         // This validates the NestedJoin recursion
//         let sql = "SELECT * FROM (users JOIN locations ON u.id = l.id)";
//         let tables = extract(sql);
//         assert_eq!(tables, vec!["users", "locations"]);
//     }

//     #[test]
//     fn test_complex_mix() {
//         // 1. Comma separation
//         // 2. Nested Join
//         // 3. Subquery (ignored)
//         let sql = "
//             SELECT *
//             FROM a,
//                  (b JOIN c ON b.id = c.id),
//                  (SELECT * FROM d) as sub
//         ";
//         let tables = extract(sql);

//         // 'a' comes from first comma part
//         // 'b', 'c' come from the nested join
//         // 'd' is ignored because it is inside a derived table (subquery)
//         assert_eq!(tables, vec!["a", "b", "c"]);
//     }

//     #[test]
//     fn test_outer_join() {
//         let sql = "SELECT Customers.CustomerName, Orders.OrderID
//                         FROM Customers
//                         FULL OUTER JOIN Orders ON Customers.CustomerID=Orders.CustomerID
//                         ORDER BY Customers.CustomerName;
//                         ";
//         let tables = extract(sql);
//         assert_eq!(tables, vec!["Customers", "Orders"])
//     }
// }
