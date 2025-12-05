use sqlparser::{
    ast::{
        Cte, Expr, JoinOperator, ObjectNamePart, SelectItem, SelectItemQualifiedWildcardKind,
        SetExpr, SetOperator, Statement, TableFactor,
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
) -> Result<Vec<ColumnInfo>, String> {
    let dialect = SQLiteDialect {};
    let sql = pg_cast_syntax_to_sqlite(sql);

    let ast = Parser::parse_sql(&dialect, &sql).map_err(|e| e.to_string())?;

    if let Statement::Query(query) = &ast[0] {
        let mut context_tables = all_tables.clone();

        if let Some(with) = &query.with {
            for cte in &with.cte_tables {
                let cte_name = cte.alias.name.value.clone();

                let inferred_cols = resolve_cte_columns(cte, &context_tables)?;

                let final_cols = inferred_cols
                    .into_iter()
                    .enumerate()
                    .map(|(i, mut col)| {
                        // If an explicit alias exists at this index, overwrite the name
                        if let Some(alias) = cte.alias.columns.get(i) {
                            col.name = alias.name.value.clone();
                        }
                        col
                    })
                    .collect();

                context_tables.insert(cte_name.to_lowercase(), final_cols);
            }
        }

        return traverse_select_output(&query.body, &context_tables);
    }

    Ok(vec![])
}


pub fn traverse_select_output(
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
                    // Current table (Right side of the join syntax) becomes nullable
                    // if it is a LEFT or FULL join.
                    let make_current_table_nullable = matches!(
                        &join.join_operator,
                        JoinOperator::Left(_)
                            | JoinOperator::LeftOuter(_)
                            | JoinOperator::FullOuter(_)
                    );

                    // Existing tables (Left side of the join syntax) become nullable
                    // if it is a RIGHT or FULL join.
                    let make_existing_tables_nullable = matches!(
                        &join.join_operator,
                        JoinOperator::Right(_)
                            | JoinOperator::RightOuter(_)
                            | JoinOperator::FullOuter(_)
                    );

                    if make_existing_tables_nullable {
                        for table_name in &local_scope_tables {
                            if let Some(cols) = working_tables.get_mut(table_name) {
                                for col in cols {
                                    col.data_type.nullable = true;
                                }
                            }
                        }
                    }

                    resolve_table_factor(
                        &join.relation,
                        make_current_table_nullable,
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

                let merged_cols = left_cols
                    .into_iter()
                    .zip(right_cols)
                    .map(|(mut l_col, r_col)| {
                        if r_col.data_type.nullable {
                            l_col.data_type.nullable = true;
                        }

                        let l_base = l_col.data_type.base_type;
                        let r_base = r_col.data_type.base_type;

                        if l_base != r_base
                            && ((l_base == BaseType::Integer && r_base == BaseType::Real)
                                || (l_base == BaseType::Real && r_base == BaseType::Integer))
                        {
                            l_col.data_type.base_type = BaseType::Real;
                        }
                        l_col
                    })
                    .collect();

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

fn resolve_cte_columns(
    cte: &Cte,
    context: &HashMap<String, Vec<ColumnInfo>>,
) -> Result<Vec<ColumnInfo>, String> {
    if let SetExpr::SetOperation { left, right, .. } = &*cte.query.body {
        let anchor_cols = traverse_select_output(left, context)?;

        let mut recursive_context = context.clone();
        let cte_name = cte.alias.name.value.clone();
        let mut context_anchor_cols = Vec::new();
        for (i, col) in anchor_cols.iter().enumerate() {
            let mut new_col = col.clone();
            if let Some(alias) = cte.alias.columns.get(i) {
                new_col.name = alias.name.value.clone();
            }
            context_anchor_cols.push(new_col);
        }
        recursive_context.insert(cte_name, context_anchor_cols);

        let recursive_cols = traverse_select_output(right, &recursive_context)?;

        // zip the anchor columns with the recursive columns.
        // If lengths differ, zip stops at the shorter one (standard SQL behavior usually implies they match).
        let merged_cols = anchor_cols
            .into_iter()
            .zip(recursive_cols)
            .map(|(mut l_col, r_col)| {
                if r_col.data_type.nullable {
                    l_col.data_type.nullable = true;
                }

                //  Numeric Types promotion (Int + Real => Real)
                let l_base = l_col.data_type.base_type;
                let r_base = r_col.data_type.base_type;

                if (l_base == BaseType::Integer && r_base == BaseType::Real)
                    || (l_base == BaseType::Real && r_base == BaseType::Integer)
                {
                    l_col.data_type.base_type = BaseType::Real;
                }

                l_col
            })
            .collect();
        Ok(merged_cols)
    } else {
        // Standard CTE
        traverse_select_output(&cte.query.body, context)
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
            let ObjectNamePart::Identifier(table_ident) = name.0.last().unwrap() else {
                panic!("Last part of name is not an identifier");
            };

            // If unquoted, look it up as lowercase. If quoted, strict.
            let lookup_name = if table_ident.quote_style.is_some() {
                table_ident.value.clone()
            } else {
                table_ident.value.to_lowercase()
            };

            let target_alias = if let Some(alias_node) = alias {
                alias_node.name.value.clone() // Aliases are case-insensitive references usually
            } else {
                lookup_name.clone()
            };

            if let Some(cols) = all_tables.get(&lookup_name) {
                let mut cols = cols.clone();
                if force_nullable {
                    for c in &mut cols {
                        c.data_type.nullable = true;
                    }
                }
                working_tables.insert(target_alias.to_lowercase(), cols.clone());
                local_scope_tables.push(target_alias.to_lowercase());
            }
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
