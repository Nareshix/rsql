use type_inference::select_patterns::get_types_from_select;
use type_inference::*;

use crate::{
    expr::BaseType,
    table::{ColumnInfo, create_tables},
};
use std::collections::HashMap;

#[derive(Debug, PartialEq)]

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

    #[test]
    fn test_aliased_columns_and_tables() {
        check_select_types(
            "SELECT u.name AS user_name, u.age FROM users u",
            vec![t(BaseType::Text, false), t(BaseType::Integer, true)],
        );

        check_select_types(
            "SELECT u.name AS user_name, u.age FROM users u",
            vec![t(BaseType::Text, false), t(BaseType::Integer, true)],
        );
    }

    #[test]
    fn test_implicit_cross_join_aliases() {
        check_select_types(
            "SELECT u.name, p.price FROM users u, products p",
            vec![t(BaseType::Text, false), t(BaseType::Real, false)],
        );
    }

    #[test]
    fn test_scalar_subquery() {
        check_select_types(
            "SELECT name, (SELECT COUNT(*) FROM orders WHERE user_id = users.id) as order_count FROM users",
            vec![t(BaseType::Text, false), t(BaseType::Integer, false)],
        );
    }

    #[test]
    fn test_derived_table_subquery() {
        check_select_types(
            "SELECT derived.p_id, derived.calc FROM (SELECT product_id as p_id, price * 1.2 as calc FROM products) derived",
            vec![t(BaseType::Integer, false), t(BaseType::Real, false)],
        );
    }

    #[test]
    fn test_subquery_in_where_in() {
        check_select_types(
            "SELECT name FROM users WHERE id IN (SELECT user_id FROM orders)",
            vec![t(BaseType::Text, false)],
        );
    }

    #[test]
    fn test_left_join_nullability_propagation() {
        check_select_types(
            "SELECT users.name, orders.total
             FROM users
             LEFT JOIN orders ON users.id = orders.user_id",
            vec![t(BaseType::Text, false), t(BaseType::Real, true)],
        );
    }

    #[test]
    fn test_inner_join_preserves_not_null() {
        check_select_types(
            "SELECT users.name, orders.total
             FROM users
             JOIN orders ON users.id = orders.user_id",
            vec![t(BaseType::Text, false), t(BaseType::Real, false)],
        );
    }

    #[test]
    fn test_simple_cte() {
        check_select_types(
            "WITH recent_orders AS (
                SELECT user_id, total FROM orders WHERE total > 100.0
             )
             SELECT total FROM recent_orders",
            vec![t(BaseType::Real, false)],
        );
    }

    #[test]
    fn test_cte_renaming_columns() {
        check_select_types(
            "WITH user_lookup(uid, uname) AS (
                SELECT id, name FROM users
             )
             SELECT uname FROM user_lookup WHERE uid = 1",
            vec![t(BaseType::Text, false)],
        );
    }

    #[test]
    fn test_chained_ctes() {
        check_select_types(
            "WITH
             step1 AS (SELECT id, costs FROM users),
             step2 AS (SELECT id, costs * 2 as double_cost FROM step1)
             SELECT double_cost FROM step2",
            vec![t(BaseType::Real, true)],
        );
    }

    #[test]
    fn test_union_nullability_mixing() {
        check_select_types(
            "SELECT id FROM users UNION SELECT stock FROM products",
            vec![t(BaseType::Integer, true)],
        );
    }

    #[test]
    fn test_union_type_coercion() {
        check_select_types(
            "SELECT id FROM users UNION SELECT 3.5",
            vec![t(BaseType::Real, false)],
        );
    }

    #[test]
    fn test_coalesce_makes_nullable_not_null() {
        check_select_types(
            "SELECT COALESCE(costs, 0.0) FROM users",
            vec![t(BaseType::Real, false)],
        );
    }

    #[test]
    fn test_nullif_makes_not_null_nullable() {
        check_select_types(
            "SELECT NULLIF(name, 'admin') FROM users",
            vec![t(BaseType::Text, true)],
        );
    }

    #[test]
    fn test_abs_function_propagates_null() {
        check_select_types(
            "SELECT ABS(costs) FROM users",
            vec![t(BaseType::Real, true)],
        );
    }

    #[test]
    fn test_string_functions() {
        check_select_types(
            "SELECT UPPER(name) FROM users",
            vec![t(BaseType::Text, false)],
        );
    }

    #[test]
    fn test_boolean_literals_and_comparison() {
        check_select_types("SELECT id > 0 FROM users", vec![t(BaseType::Bool, false)]);

        check_select_types(
            "SELECT TRUE, FALSE",
            vec![t(BaseType::Bool, false), t(BaseType::Bool, false)],
        );

        check_select_types(
            "SELECT (costs > 0) AND (age < 100) FROM users",
            vec![t(BaseType::Bool, true)],
        );
    }

    #[test]
    fn test_is_null_check() {
        check_select_types(
            "SELECT costs IS NULL FROM users",
            vec![t(BaseType::Bool, false)],
        );
    }

    #[test]
    fn test_exists_subquery() {
        check_select_types(
            "SELECT EXISTS(SELECT 1 FROM users WHERE id = 1)",
            vec![t(BaseType::Bool, false)],
        );
    }
    #[test]
    fn test_select_pure_placeholder() {
        check_select_types("SELECT ?", vec![t(BaseType::PlaceHolder, true)]);
    }

    #[test]
    fn test_placeholder_math_inference() {
        check_select_types("SELECT ? + 100", vec![t(BaseType::Integer, true)]);

        check_select_types("SELECT ? + 100.5", vec![t(BaseType::Real, true)]);
    }
    #[test]
    fn test_window_function_row_number() {
        check_select_types(
            "SELECT ROW_NUMBER() OVER (ORDER BY id) FROM users",
            vec![t(BaseType::Integer, false)],
        );
    }

    #[test]
    fn test_window_function_aggregates() {
        check_select_types(
            "SELECT SUM(total) OVER (PARTITION BY user_id) FROM orders",
            vec![t(BaseType::Real, true)],
        );
    }

    #[test]
    fn test_string_length() {
        check_select_types(
            "SELECT LENGTH(name), LENGTH(costs) FROM users",
            vec![t(BaseType::Integer, false), t(BaseType::Integer, true)],
        );
    }

    #[test]
    fn test_substr() {
        check_select_types(
            "SELECT SUBSTR(name, 1, 3) FROM users",
            vec![t(BaseType::Text, false)],
        );
    }

    #[test]
    fn test_except_operation() {
        check_select_types(
            "SELECT id FROM users EXCEPT SELECT stock FROM products",
            vec![t(BaseType::Integer, false)],
        );
    }

    #[test]
    fn test_recursive_cte_counter() {
        check_select_types(
            "WITH RECURSIVE cnt(x) AS (
            SELECT 1
            UNION ALL
            SELECT x+1 FROM cnt WHERE x < 10
         )
         SELECT x FROM cnt",
            vec![t(BaseType::Integer, false)],
        );
    }

    #[test]
    fn test_mixed_numeric_types() {
        check_select_types("SELECT 10 + 5.5", vec![t(BaseType::Real, false)]);
    }

    #[test]
    fn test_nested_left_join_nullability() {
        check_select_types(
            "SELECT sub.t
         FROM users
         LEFT JOIN (SELECT total as t, user_id FROM orders) sub
         ON users.id = sub.user_id",
            vec![t(BaseType::Real, true)],
        );
    }

    #[test]
    fn test_correlated_subquery() {
        check_select_types(
            "SELECT name,
         (SELECT total FROM orders WHERE user_id = u.id LIMIT 1) as last_order_total
         FROM users u",
            vec![t(BaseType::Text, false), t(BaseType::Real, true)],
        );
    }
    #[test]
    fn test_cte_shadows_table_name() {
        check_select_types(
            "WITH users AS (
            SELECT price, stock FROM products
         )
         SELECT price FROM users",
            vec![t(BaseType::Real, false)],
        );
    }
    #[test]
    fn test_deeply_nested_subquery() {
        check_select_types(
            "SELECT * FROM (
            SELECT (
                SELECT product_id FROM products WHERE price > 1000
            ) as expensive_id
        ) tmp",
            vec![t(BaseType::Integer, true)],
        );
    }
    #[test]
    fn test_cte_with_values_clause() {
        check_select_types(
            "WITH const_data(a, b) AS (
            VALUES(1, 2.5)
         )
         SELECT b FROM const_data",
            vec![t(BaseType::Real, false)],
        );
    }

    #[test]
    fn test_cte_usage_inside_derived_table() {
        check_select_types(
            "WITH raw_data AS (SELECT id, costs FROM users)
         SELECT d.double_cost
         FROM (SELECT costs * 2 AS double_cost FROM raw_data) d",
            vec![t(BaseType::Real, true)],
        );
    }
}