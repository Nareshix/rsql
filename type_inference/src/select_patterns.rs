use sqlparser::{
    ast::{ObjectNamePart, SelectItem, SelectItemQualifiedWildcardKind, SetExpr, Statement},
    dialect::SQLiteDialect,
    parser::Parser,
};
use std::collections::HashMap;

use crate::expr::evaluate_expr_type;
use crate::{
    expr::Type,
    table::{ColumnInfo, get_table_names},
};

#[allow(unused)]
pub fn get_types_from_select(
    sql: &str,
    all_tables: &HashMap<String, Vec<ColumnInfo>>,
) -> Vec<Result<Type, String>> {
    let dialect = SQLiteDialect {};
    let table_names_from_select = get_table_names(sql);

    let ast = Parser::parse_sql(&dialect, sql).unwrap();

    let mut column_types = Vec::new();

    if let Statement::Query(query) = &ast[0]
        && let SetExpr::Select(select) = &*query.body
    {
        for item in &select.projection {
            match item {
                // SELECT column_name OR SELECT count(*) and other aggregate fns
                SelectItem::UnnamedExpr(expr) => {
                    let t = evaluate_expr_type(expr, &table_names_from_select, all_tables);
                    column_types.push(t);
                }

                // SELECT column_name AS alias
                SelectItem::ExprWithAlias { expr, alias: _ } => {
                    let t = evaluate_expr_type(expr, &table_names_from_select, all_tables);
                    column_types.push(t);
                }

                // SELECT *
                SelectItem::Wildcard(_) => {
                    for table_name in &table_names_from_select {
                        let column_infos = &all_tables[table_name];
                        for column_info in column_infos {
                            column_types.push(Ok(column_info.data_type.clone()));
                        }
                    }
                }

                // SELECT users.*
                SelectItem::QualifiedWildcard(object_name, _) => match object_name {
                    SelectItemQualifiedWildcardKind::ObjectName(obj) => {
                        if let ObjectNamePart::Identifier(ident) = &obj.0[0] {
                            // its guranteed to be only one table and we are getting
                            // all the col type from that table
                            let table_name = &ident.value;
                            let column_infos = &all_tables[table_name];
                            for column_info in column_infos {
                                column_types.push(Ok(column_info.data_type.clone()));
                            }
                        }
                    }
                    SelectItemQualifiedWildcardKind::Expr(expr) => {
                        let t =
                            evaluate_expr_type(expr, &table_names_from_select, all_tables);
                        column_types.push(t);
                    }
                },
            }
        }
    }

    column_types
}
