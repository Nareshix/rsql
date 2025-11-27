use sqlparser::{
    ast::{SelectItem, SetExpr, Statement},
    dialect::SQLiteDialect,
    parser::Parser,
};
use std::collections::HashMap;

// Assuming these exist in your crate
use crate::expr::evaluate_expr_type;
use crate::{
    expr::Type,
    table::{ColumnInfo, get_table_names},
};

#[allow(unused)]
pub fn get_types_from_select(
    sql: &str,
    all_tables: &HashMap<String, Vec<ColumnInfo>>,
) -> Vec<Type> {
    let dialect = SQLiteDialect {};
    let table_names_from_select = get_table_names(sql);

    let ast = Parser::parse_sql(&dialect, sql).unwrap();

    let mut column_types = Vec::new();

    if let Statement::Query(query) = &ast[0]
        && let SetExpr::Select(select) = &*query.body
    {
        for item in &select.projection {
            match item {
                // SELECT column_name OR SELECT count(*)
                SelectItem::UnnamedExpr(expr) => {
                    let t = evaluate_expr_type(expr, table_names_from_select.clone(), all_tables);
                    column_types.push(t);
                }

                // Case: SELECT column_name AS alias
                SelectItem::ExprWithAlias { expr, alias: _ } => {
                    let t = evaluate_expr_type(expr, table_names_from_select.clone(), all_tables);
                    column_types.push(t);
                }

                // Case: SELECT *
                SelectItem::Wildcard(_options) => {
                    todo!();
                }

                // Case: SELECT users.*
                SelectItem::QualifiedWildcard(object_name, _options) => {
                    todo!();
                }
            }
        }
    }

    column_types
}
