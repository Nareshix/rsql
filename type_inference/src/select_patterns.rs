use sqlparser::{
    ast::{SelectItem, SelectItemQualifiedWildcardKind, SetExpr, Statement},
    dialect::SQLiteDialect,
    parser::Parser,
};
use std::collections::HashMap;

use crate::expr::evaluate_expr_type;
use crate::{expr::Type, table::ColumnInfo};

pub fn get_types_from_select(
    sql: &str,
    all_tables: &HashMap<String, Vec<ColumnInfo>>,
) -> Vec<Result<Type, String>> {
    let dialect = SQLiteDialect {};
    let ast = Parser::parse_sql(&dialect, sql).unwrap();

    if let Statement::Query(query) = &ast[0] {
        let mut context_tables = all_tables.clone();

        if let Some(with) = &query.with {
            for cte in &with.cte_tables {
                let cte_name = cte.alias.name.value.clone();

                let cte_types = if let SetExpr::SetOperation { left, .. } = &*cte.query.body {
                    traverse_select_output(left, &context_tables)
                } else {
                    traverse_select_output(&cte.query.body, &context_tables)
                };

                let mut cols = Vec::new();
                for (i, t_res) in cte_types.into_iter().enumerate() {
                    if let Ok(t) = t_res {
                        let col_name = if let Some(col_def) = cte.alias.columns.get(i) {
                            col_def.name.value.clone()
                        }
                        else {
                            format!("col_{}", i)
                        };

                        cols.push(ColumnInfo {
                            name: col_name,
                            data_type: t,
                            check_constraint: None,
                        });
                    }
                }
                context_tables.insert(cte_name, cols);
            }
        }

        return traverse_select_output(&query.body, &context_tables);
    }

    vec![]
}

fn traverse_select_output(
    body: &SetExpr,
    all_tables: &HashMap<String, Vec<ColumnInfo>>,
) -> Vec<Result<Type, String>> {
    match body {
        SetExpr::Select(select) => {
            let mut local_scope_tables = Vec::new();
            let mut working_tables = all_tables.clone();

            for table_with_joins in &select.from {
                let mut relations = vec![&table_with_joins.relation];
                for join in &table_with_joins.joins {
                    relations.push(&join.relation);
                }

                for relation in relations {
                    match relation {
                        // Case A: Standard Table
                        sqlparser::ast::TableFactor::Table { name, alias, .. } => {
                            let real_name =
                                if let Some(sqlparser::ast::ObjectNamePart::Identifier(ident)) =
                                    name.0.last()
                                {
                                    ident.value.clone()
                                } else {
                                    name.to_string()
                                };

                            if let Some(alias_node) = alias {
                                let alias_name = alias_node.name.value.clone();
                                if let Some(cols) = all_tables.get(&real_name) {
                                    working_tables.insert(alias_name.clone(), cols.clone());
                                }
                                local_scope_tables.push(alias_name);
                            } else {
                                local_scope_tables.push(real_name);
                            }
                        }

                        sqlparser::ast::TableFactor::Derived {
                            subquery,
                            alias: Some(alias_node),
                            ..
                        } => {
                            let alias_name = alias_node.name.value.clone();

                            let sub_types = traverse_select_output(&subquery.body, all_tables);

                            let mut cols = Vec::new();
                            for (i, t_res) in sub_types.into_iter().enumerate() {
                                if let Ok(t) = t_res {
                                    let col_name = if let SetExpr::Select(sub_s) = &*subquery.body
                                        && let Some(SelectItem::ExprWithAlias { alias, .. }) =
                                            sub_s.projection.get(i)
                                    {
                                        alias.value.clone()
                                    } else {
                                        format!("col_{}", i)
                                    };

                                    cols.push(ColumnInfo {
                                        name: col_name,
                                        data_type: t,
                                        check_constraint: None,
                                    });
                                }
                            }

                            working_tables.insert(alias_name.clone(), cols);
                            local_scope_tables.push(alias_name);
                        }
                        _ => {}
                    }
                }
            }

            // 2. Process Projections
            let mut column_types = Vec::new();

            for item in &select.projection {
                match item {
                    SelectItem::UnnamedExpr(expr) | SelectItem::ExprWithAlias { expr, .. } => {
                        let t = evaluate_expr_type(expr, &local_scope_tables, &working_tables);
                        column_types.push(t);
                    }

                    SelectItem::Wildcard(_) => {
                        for table_name in &local_scope_tables {
                            if let Some(column_infos) = working_tables.get(table_name) {
                                for column_info in column_infos {
                                    column_types.push(Ok(column_info.data_type.clone()));
                                }
                            }
                        }
                    }

                    SelectItem::QualifiedWildcard(kind, _) => {
                        if let SelectItemQualifiedWildcardKind::ObjectName(obj_name) = kind {
                            let alias_name =
                                if let Some(sqlparser::ast::ObjectNamePart::Identifier(ident)) =
                                    obj_name.0.last()
                                {
                                    ident.value.clone()
                                } else {
                                    obj_name.to_string()
                                };

                            if let Some(column_infos) = working_tables.get(&alias_name) {
                                for column_info in column_infos {
                                    column_types.push(Ok(column_info.data_type.clone()));
                                }
                            }
                        }
                    }
                }
            }
            column_types
        }

        SetExpr::SetOperation { left, .. } => traverse_select_output(left, all_tables),

        SetExpr::Values(values) => {
            let mut types = Vec::new();
            if let Some(first_row) = values.rows.first() {
                for expr in first_row {
                    types.push(evaluate_expr_type(expr, &vec![], all_tables));
                }
            }
            types
        }

        SetExpr::Query(q) => traverse_select_output(&q.body, all_tables),

        _ => vec![],
    }
}