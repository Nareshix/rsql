use std::collections::HashMap;

use sqlparser::ast::{ColumnOption, CreateTable, Query, SetExpr, Statement, TableFactor};
use sqlparser::dialect::SQLiteDialect;
use sqlparser::parser::Parser;

use crate::expr::Type;

// use crate::expr::Type;

// 1. The Structs
#[derive(Debug, Clone, PartialEq)]
#[allow(unused)]
pub struct FieldInfo {
    pub name: String,
    pub data_type: Type,
    pub check_constraint: Option<String>,
}

#[derive(Debug)]
#[allow(unused)]
pub struct TableSchema {
    pub table_name: String,
    pub fields: Vec<FieldInfo>,
}

#[allow(unused)]
fn convert_sqlite_to_rust_type(sql: String) -> Type {
    if sql == "TEXT" {
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

#[allow(unused)]
/// parses the sql and creates an ast for table. then it  is inserted into the Hashmap.
pub fn create_tables(sql: &str, tables: &mut HashMap<String, Vec<FieldInfo>>) {
    let dialect = SQLiteDialect {};
    let ast = Parser::parse_sql(&dialect, sql).unwrap();

    let schema = if let Statement::CreateTable(CreateTable { name, columns, .. }) = &ast[0] {
        TableSchema {
            table_name: name.to_string(),
            fields: columns
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
                    FieldInfo {
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
    tables.insert(schema.table_name, schema.fields);
}


#[allow(unused)]
/// does not support ctes and subqueries
pub fn get_table_names(query: &Query) -> Vec<String> {
    let mut table_names = Vec::new();

    // ignore  query.with (CTEs)
    if let SetExpr::Select(select) = &*query.body {
        // iterate over FROM clause
        for table_with_joins in &select.from {

            // Check the main table in the FROM clause
            extract_table(&table_with_joins.relation, &mut table_names);

            // Iterate over any JOINs attached to this table
            for join in &table_with_joins.joins {
                extract_table(&join.relation, &mut table_names);
            }
        }
    }

    table_names
}

fn extract_table(relation: &TableFactor, table_names: &mut Vec<String>) {
    match relation {
        TableFactor::Table { name, .. } => {
            table_names.push(name.to_string());
        }

        // FROM (SELECT ...), ignore subqueries
        TableFactor::Derived { .. } => {}

        // Handle nested joins 
        TableFactor::NestedJoin{table_with_joins, ..} => {
            extract_table(&table_with_joins.relation, table_names);
            // handles flatteed joins
            for join in &table_with_joins.joins {
                extract_table(&join.relation, table_names);
            }
        }
        _ => {}
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    // Helper to parse SQL quickly
    fn extract(sql: &str) -> Vec<String> {
        let dialect = SQLiteDialect {};
        let ast = Parser::parse_sql(&dialect, sql).unwrap();
        match &ast[0] {
            sqlparser::ast::Statement::Query(q) => get_table_names(q),
            _ => vec![],
        }
    }

    #[test]
    fn test_basic_tables() {
        let tables = extract("SELECT * FROM users");
        assert_eq!(tables, vec!["users"]);
    }

    #[test]
    fn test_comma_separated() {
        // Handles "FROM A, B"
        let tables = extract("SELECT * FROM users, accounts");
        assert_eq!(tables, vec!["users", "accounts"]);
    }

    #[test]
    fn test_standard_joins() {
        // Handles "FROM A JOIN B JOIN C"
        let tables = extract("
            SELECT * 
            FROM users 
            JOIN orders ON users.id = orders.uid
            LEFT JOIN items ON orders.id = items.oid
        ");
        assert_eq!(tables, vec!["users", "orders", "items"]);
    }

    #[test]
    fn test_ignore_ctes() {
        // Should NOT find "cte_table"
        // Should find "real_table"
        let sql = "
            WITH cte_table AS (SELECT * FROM hidden) 
            SELECT * FROM real_table
        ";
        let tables = extract(sql);
        assert_eq!(tables, vec!["real_table"]);
    }

    #[test]
    fn test_ignore_subqueries() {
        // Should NOT find "hidden" inside the subquery
        // Should find "users"
        let sql = "
            SELECT * 
            FROM users 
            JOIN (SELECT * FROM hidden) AS sub ON sub.id = users.id
        ";
        let tables = extract(sql);
        assert_eq!(tables, vec!["users"]);
    }


    #[test]
    fn test_nested_joins() {
        // Handles parentheses: FROM (A JOIN B)
        // This validates the NestedJoin recursion
        let sql = "SELECT * FROM (users JOIN locations ON u.id = l.id)";
        let tables = extract(sql);
        assert_eq!(tables, vec!["users", "locations"]);
    }

    #[test]
    fn test_complex_mix() {
        // 1. Comma separation
        // 2. Nested Join
        // 3. Subquery (ignored)
        let sql = "
            SELECT * 
            FROM a, 
                 (b JOIN c ON b.id = c.id), 
                 (SELECT * FROM d) as sub
        ";
        let tables = extract(sql);
        
        // 'a' comes from first comma part
        // 'b', 'c' come from the nested join
        // 'd' is ignored because it is inside a derived table (subquery)
        assert_eq!(tables, vec!["a", "b", "c"]);
    }
}
