use type_inference::select_patterns::get_types_from_select;
use type_inference::*;

use crate::{
    expr::BaseType,
    table::{ColumnInfo, create_tables},
};
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
// we do not care whether it contains placeholder or not. its just for internal use
struct ColumnType {
    base_type: BaseType,
    nullable: bool,
}

fn setup_tables() -> HashMap<String, Vec<ColumnInfo>> {
    let mut tables = HashMap::new();

    create_tables(
        "CREATE TABLE users (id INTEGER NOT NULL, name TEXT NOT NULL, costs REAL, age INTEGER)",
        &mut tables,
    );

    create_tables(
        "CREATE TABLE products (product_id INTEGER NOT NULL, price REAL NOT NULL, stock INTEGER, description TEXT)",
        &mut tables,
    );

    create_tables(
        "CREATE TABLE orders (order_id INTEGER NOT NULL, user_id INTEGER NOT NULL, total REAL NOT NULL, created_at TEXT NOT NULL)",
        &mut tables,
    );
    tables
}

#[track_caller]
fn check_select_types(sql: &str, expected: Vec<ColumnType>) {
    let tables = setup_tables();

    let internal_types = get_types_from_select(sql, &tables).unwrap();

    let select_types: Vec<ColumnType> = internal_types
        .into_iter()
        .map(|t| ColumnType {
            base_type: t.base_type,
            nullable: t.nullable,
        })
        .collect();

    assert_eq!(select_types, expected);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(base: BaseType, nullable: bool) -> ColumnType {
        ColumnType {
            base_type: base,
            nullable,
        }
    }

    #[test]
    fn test_select_simple_columns() {
        check_select_types(
            "SELECT id, name FROM users",
            vec![t(BaseType::Integer, false), t(BaseType::Text, false)],
        );
    }

    #[test]
    fn test_select_wildcard() {
        check_select_types(
            "SELECT * FROM products",
            vec![
                t(BaseType::Integer, false),
                t(BaseType::Real, false),
                t(BaseType::Integer, true),
                t(BaseType::Text, true),
            ],
        );
    }

    #[test]
    fn test_select_literals() {
        check_select_types(
            "SELECT 1, 'hello', 3.14 FROM users",
            vec![
                t(BaseType::Integer, false),
                t(BaseType::Text, false),
                t(BaseType::Real, false),
            ],
        );
    }

    #[test]
    fn test_expression_math_not_null() {
        check_select_types(
            "SELECT price * 2 FROM products",
            vec![t(BaseType::Real, false)],
        );
    }

    #[test]
    fn test_expression_math_nullable_mix() {
        check_select_types(
            "SELECT stock + 10 FROM products",
            vec![t(BaseType::Integer, true)],
        );
    }

    #[test]
    fn test_expression_string_concat() {
        check_select_types(
            "SELECT name || ' suffix' FROM users",
            vec![t(BaseType::Text, false)],
        );
    }

    #[test]
    fn test_expression_string_concat_nullable() {
        check_select_types(
            "SELECT description || ' suffix' FROM products",
            vec![t(BaseType::Text, true)],
        );
    }

    #[test]
    fn test_basic_where_binding() {
        check_select_types(
            "SELECT name FROM users WHERE id = ?",
            vec![t(BaseType::Text, false)],
        );
    }

    #[test]
    fn test_multiple_where_bindings() {
        check_select_types(
            "SELECT order_id FROM orders WHERE user_id = ? AND total > ?",
            vec![t(BaseType::Integer, false)],
        );
    }

    #[test]
    fn test_insert_inference() {
        check_select_types("INSERT INTO users (name, age) VALUES (?, ?)", vec![]);
    }

    #[test]
    fn test_update_inference() {
        check_select_types("UPDATE users SET costs = ? WHERE id = ?", vec![]);
    }

    #[test]
    fn test_delete_inference() {
        check_select_types("DELETE FROM products WHERE product_id = ?", vec![]);
    }

    #[test]
    fn test_binary_op_promotion_and_binding() {
        check_select_types(
            "SELECT costs + 10 FROM users WHERE age > ?",
            vec![t(BaseType::Real, true)],
        );
    }

    #[test]
    fn test_complex_math_expression() {
        check_select_types(
            "SELECT price * stock FROM products",
            vec![t(BaseType::Real, true)],
        );
    }

    #[test]
    fn test_like_inference() {
        check_select_types(
            "SELECT id FROM users WHERE name LIKE ?",
            vec![t(BaseType::Integer, false)],
        );
    }

    #[test]
    fn test_in_list_inference() {
        check_select_types(
            "SELECT name FROM users WHERE id IN (?, ?, 5)",
            vec![t(BaseType::Text, false)],
        );
    }

    #[test]
    fn test_aggregates_count() {
        check_select_types(
            "SELECT COUNT(id) FROM users",
            vec![t(BaseType::Integer, false)],
        );
    }

    #[test]
    fn test_aggregates_avg() {
        check_select_types("SELECT AVG(age) FROM users", vec![t(BaseType::Real, true)]);
    }

    #[test]
    fn test_aggregates_sum() {
        check_select_types(
            "SELECT SUM(total) FROM orders",
            vec![t(BaseType::Real, true)],
        );
    }

    #[test]
    fn test_explicit_cast_binding() {
        check_select_types("SELECT CAST(? AS TEXT)", vec![t(BaseType::Text, true)]);
    }

    #[test]
    fn test_explicit_cast_output() {
        check_select_types(
            "SELECT CAST(age AS REAL) FROM users",
            vec![t(BaseType::Real, true)],
        );
    }

    #[test]
    fn test_explicit_cast_output_not_null() {
        check_select_types(
            "SELECT CAST(id AS REAL) FROM users",
            vec![t(BaseType::Real, false)],
        );
    }

    #[test]
    fn test_join_column_resolution() {
        check_select_types(
            "SELECT products.description, orders.total
             FROM products
             JOIN orders ON products.product_id = orders.order_id",
            vec![t(BaseType::Text, true), t(BaseType::Real, false)],
        );
    }

    #[test]
    fn test_case_statement() {
        check_select_types(
            "SELECT CASE WHEN costs > ? THEN 'High' ELSE 'Low' END FROM users",
            vec![t(BaseType::Text, false)],
        );
    }

    #[test]
    fn test_case_statement_with_null() {
        check_select_types(
            "SELECT CASE WHEN stock > 0 THEN description ELSE 'No Stock' END FROM products",
            vec![t(BaseType::Text, true)],
        );
    }

    #[test]
    fn test_between_inference() {
        check_select_types(
            "SELECT name FROM users WHERE age BETWEEN ? AND ?",
            vec![t(BaseType::Text, false)],
        );
    }

}
