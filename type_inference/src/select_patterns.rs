use sqlparser::{
    ast::{
        Expr, JoinOperator, SelectItem, SelectItemQualifiedWildcardKind, SetExpr, SetOperator,
        Statement, TableFactor,
    },
    dialect::SQLiteDialect,
    parser::Parser,
};
use std::collections::HashMap;

use crate::expr::{BaseType, evaluate_expr_type};
use crate::pg_type_cast_to_sqlite::pg_cast_syntax_to_sqlite;
use crate::{expr::Type, table::ColumnInfo};
pub fn get_types_from_select(
    sql: &str,
    all_tables: &HashMap<String, Vec<ColumnInfo>>,
) -> Result<Vec<Type>, String> {
    let dialect = SQLiteDialect {};
    let sql = pg_cast_syntax_to_sqlite(sql);

    let ast = Parser::parse_sql(&dialect, &sql).map_err(|e| e.to_string())?;

    if let Statement::Query(query) = &ast[0] {
        let mut context_tables = all_tables.clone();

        if let Some(with) = &query.with {
            for cte in &with.cte_tables {
                let cte_name = cte.alias.name.value.clone();

                // 1. Check if the body is a UNION (SetOperation)
                let (mut final_cols, right_body_opt) =
                    if let SetExpr::SetOperation { left, right, .. } = &*cte.query.body {
                        // Analyze LEFT side (Anchor)
                        let left_cols = traverse_select_output(left, &context_tables)?;
                        (left_cols, Some(right))
                    } else {
                        // Standard CTE
                        (
                            traverse_select_output(&cte.query.body, &context_tables)?,
                            None,
                        )
                    };

                // 2. Prepare the Initial Context for the Recursive step
                // We must register the CTE now so the Recursive part can "see" itself.
                let mut initial_cte_cols = Vec::new();
                for (i, col) in final_cols.iter().enumerate() {
                    let name = if let Some(col_def) = cte.alias.columns.get(i) {
                        col_def.name.value.clone()
                    } else {
                        col.name.clone()
                    };
                    initial_cte_cols.push(ColumnInfo {
                        name,
                        data_type: col.data_type.clone(),
                        check_constraint: None,
                    });
                }
                context_tables.insert(cte_name.clone(), initial_cte_cols.clone());

                // 3. If it was a UNION, now analyze the RIGHT side (Recursive)
                if let Some(right_body) = right_body_opt {
                    // This call uses 'context_tables' which now contains the CTE definition based on Left
                    let right_cols = traverse_select_output(right_body, &context_tables)?;

                    // 4. Merge/Promote types (Int + Real = Real)
                    let mut promoted_cols = Vec::new();
                    for (i, l_col) in final_cols.iter().enumerate() {
                        let mut final_col = l_col.clone();
                        if let Some(r_col) = right_cols.get(i) {
                            // Nullability
                            if r_col.data_type.nullable {
                                final_col.data_type.nullable = true;
                            }
                            // Promotion
                            if l_col.data_type.base_type == BaseType::Integer
                                && r_col.data_type.base_type == BaseType::Real
                            {
                                final_col.data_type.base_type = BaseType::Real;
                            }
                            // Note: If you have Text + Int, SQLite prefers Text usually,
                            // but for now Int+Real is the critical one.
                        }
                        promoted_cols.push(final_col);
                    }

                    // 5. Update the context with the PROMOTED types
                    final_cols = promoted_cols; // Update for final registration

                    // Re-register with new types
                    let mut final_cte_cols = Vec::new();
                    for (i, col) in final_cols.iter().enumerate() {
                        let name = if let Some(col_def) = cte.alias.columns.get(i) {
                            col_def.name.value.clone()
                        } else {
                            col.name.clone()
                        };
                        final_cte_cols.push(ColumnInfo {
                            name,
                            data_type: col.data_type.clone(),
                            check_constraint: None,
                        });
                    }
                    context_tables.insert(cte_name, final_cte_cols);
                }
            }
        }

        let final_cols = traverse_select_output(&query.body, &context_tables)?;
        return Ok(final_cols.into_iter().map(|c| c.data_type).collect());
    }

    Ok(vec![])
}
fn traverse_select_output(
    body: &SetExpr,
    all_tables: &HashMap<String, Vec<ColumnInfo>>,
) -> Result<Vec<ColumnInfo>, String> {
    match body {
        SetExpr::Select(select) => {
            let mut local_scope_tables = Vec::new();
            let mut working_tables = all_tables.clone();

            for table_with_joins in &select.from {
                resolve_table_factor(
                    &table_with_joins.relation,
                    false,
                    all_tables,
                    &mut working_tables,
                    &mut local_scope_tables,
                )?;

                for join in &table_with_joins.joins {
                    let is_join_nullable = matches!(
                        &join.join_operator,
                        JoinOperator::Left(_)
                            | JoinOperator::LeftOuter(_)
                            | JoinOperator::Right(_)
                            | JoinOperator::RightOuter(_)
                            | JoinOperator::FullOuter(_)
                    );

                    resolve_table_factor(
                        &join.relation,
                        is_join_nullable,
                        all_tables,
                        &mut working_tables,
                        &mut local_scope_tables,
                    )?;
                }
            }

            let mut output_columns = Vec::new();

            for (i, item) in select.projection.iter().enumerate() {
                match item {
                    SelectItem::ExprWithAlias { expr, alias } => {
                        let t = evaluate_expr_type(expr, &local_scope_tables, &working_tables)?;
                        output_columns.push(ColumnInfo {
                            name: alias.value.clone(),
                            data_type: t,
                            check_constraint: None,
                        });
                    }
                    SelectItem::UnnamedExpr(expr) => {
                        let t = evaluate_expr_type(expr, &local_scope_tables, &working_tables)?;
                        let name = match expr {
                            Expr::Identifier(ident) => ident.value.clone(),
                            Expr::CompoundIdentifier(idents) => {
                                idents.last().unwrap().value.clone()
                            }
                            _ => format!("col_{}", i),
                        };
                        output_columns.push(ColumnInfo {
                            name,
                            data_type: t,
                            check_constraint: None,
                        });
                    }
                    SelectItem::Wildcard(_) => {
                        for table_name in &local_scope_tables {
                            if let Some(column_infos) = working_tables.get(table_name) {
                                for column_info in column_infos {
                                    output_columns.push(column_info.clone());
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
                                    output_columns.push(column_info.clone());
                                }
                            }
                        }
                    }
                }
            }
            Ok(output_columns)
        }

        SetExpr::SetOperation {
            op, left, right, ..
        } => {
            let left_cols = traverse_select_output(left, all_tables)?;

            if matches!(op, SetOperator::Union) {
                let right_cols = traverse_select_output(right, all_tables)?;

                let mut merged_cols = Vec::new();
                for (i, l_col) in left_cols.iter().enumerate() {
                    let mut final_col = l_col.clone();

                    if let Some(r_col) = right_cols.get(i) {
                        if r_col.data_type.nullable {
                            final_col.data_type.nullable = true;
                        }

                        let l_base = l_col.data_type.base_type;
                        let r_base = r_col.data_type.base_type;

                        if l_base != r_base
                            && ((l_base == BaseType::Integer && r_base == BaseType::Real)
                                || (l_base == BaseType::Real && r_base == BaseType::Integer))
                        {
                            final_col.data_type.base_type = BaseType::Real;
                        }
                    }
                    merged_cols.push(final_col);
                }
                Ok(merged_cols)
            } else {
                Ok(left_cols)
            }
        }

        SetExpr::Values(values) => {
            let mut cols = Vec::new();
            if let Some(first_row) = values.rows.first() {
                for (i, expr) in first_row.iter().enumerate() {
                    let t = evaluate_expr_type(expr, &vec![], all_tables)?;
                    cols.push(ColumnInfo {
                        name: format!("col_{}", i),
                        data_type: t,
                        check_constraint: None,
                    });
                }
            }
            Ok(cols)
        }

        SetExpr::Query(q) => traverse_select_output(&q.body, all_tables),

        _ => Ok(vec![]),
    }
}

fn resolve_table_factor(
    relation: &TableFactor,
    force_nullable: bool,
    all_tables: &HashMap<String, Vec<ColumnInfo>>,
    working_tables: &mut HashMap<String, Vec<ColumnInfo>>,
    local_scope_tables: &mut Vec<String>,
) -> Result<(), String> {
    match relation {
        TableFactor::Table { name, alias, .. } => {
            let real_name =
                if let Some(sqlparser::ast::ObjectNamePart::Identifier(ident)) = name.0.last() {
                    ident.value.clone()
                } else {
                    name.to_string()
                };

            let target_alias = if let Some(alias_node) = alias {
                alias_node.name.value.clone()
            } else {
                real_name.clone()
            };

            if let Some(cols) = all_tables.get(&real_name) {
                let mut cols = cols.clone();
                if force_nullable {
                    for c in &mut cols {
                        c.data_type.nullable = true;
                    }
                }
                working_tables.insert(target_alias.clone(), cols);
            }
            local_scope_tables.push(target_alias);
        }

        TableFactor::Derived {
            subquery,
            alias: Some(alias_node),
            ..
        } => {
            let alias_name = alias_node.name.value.clone();
            let mut sub_cols = traverse_select_output(&subquery.body, all_tables)?;

            if force_nullable {
                for c in &mut sub_cols {
                    c.data_type.nullable = true;
                }
            }

            working_tables.insert(alias_name.clone(), sub_cols);
            local_scope_tables.push(alias_name);
        }
        _ => {}
    }
    Ok(())
}
