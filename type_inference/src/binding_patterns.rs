use crate::expr::{BaseType, Type, evaluate_expr_type};
use crate::table::{ColumnInfo, get_table_names};
use sqlparser::ast::{
    BinaryOperator, DataType, Expr, FunctionArg, FunctionArgExpr, FunctionArguments, Statement,
};
use sqlparser::dialect::SQLiteDialect;
use sqlparser::parser::Parser;
use std::collections::HashMap;

#[allow(unused)]
pub fn get_type_of_binding_parameters(
    sql: &str,
    all_tables: &mut HashMap<String, Vec<ColumnInfo>>,
) -> Result<Vec<Type>, String> {
    let statement = &Parser::parse_sql(&SQLiteDialect {}, sql).map_err(|e| e.to_string())?[0];

    let mut working_tables = all_tables.clone();
    let table_names = get_table_names(sql);
    let mut results = Vec::new();

    match statement {
        Statement::Query(q) => {
            traverse_query(q, &table_names, &mut working_tables, &mut results)?;
        }

        Statement::Delete(delete_node) => {
            if let Some(expr) = &delete_node.selection {
                traverse_expr(expr, &table_names, all_tables, &mut results, None)?;
            }
        }

        Statement::Update {
            assignments,
            selection,
            table,
            ..
        } => {
            let table_name = table.relation.to_string();

            for assignment in assignments {
                let col_name = match &assignment.target {
                    sqlparser::ast::AssignmentTarget::ColumnName(obj_name) => {
                        obj_name.0.last().map(|p| p.to_string()).unwrap_or_default()
                    }
                    _ => String::new(), // Skip tuple assignments for now
                };

                let mut hint = None;
                if !col_name.is_empty()
                    && let Some(cols) = all_tables.get(&table_name)
                    && let Some(col_info) = cols.iter().find(|c| c.name == col_name)
                {
                    hint = Some(col_info.data_type.clone());
                }

                traverse_expr(
                    &assignment.value,
                    &table_names,
                    all_tables,
                    &mut results,
                    hint,
                )?;
            }

            // WHERE Clause
            if let Some(expr) = selection {
                traverse_expr(expr, &table_names, all_tables, &mut results, None)?;
            }
        }

        Statement::Insert(insert_node) => {
            let t_name = match &insert_node.table {
                sqlparser::ast::TableObject::TableName(obj_name) => obj_name.to_string(),
                _ => String::new(),
            };

            let expected_types = if let Some(table_cols) = all_tables.get(&t_name) {
                if insert_node.columns.is_empty() {
                    // Implicit: Use all columns in definition order
                    table_cols
                        .iter()
                        .map(|c| Some(c.data_type.clone()))
                        .collect()
                } else {
                    // Explicit: Match provided column identifiers
                    insert_node
                        .columns
                        .iter()
                        .map(|ident| {
                            table_cols
                                .iter()
                                .find(|c| c.name == ident.value)
                                .map(|c| c.data_type.clone())
                        })
                        .collect()
                }
            } else {
                Vec::new()
            };

            if let Some(source_query) = &insert_node.source {
                match &*source_query.body {
                    // Case A: INSERT ... VALUES (...)
                    sqlparser::ast::SetExpr::Values(values) => {
                        for row in &values.rows {
                            for (idx, expr) in row.iter().enumerate() {
                                let hint = expected_types.get(idx).cloned().flatten();
                                traverse_expr(expr, &table_names, all_tables, &mut results, hint)?;
                            }
                        }
                    }
                    // Case B: INSERT ... SELECT ...
                    sqlparser::ast::SetExpr::Select(select) => {
                        // We need to map the expected types to the projection columns of the SELECT
                        for (idx, item) in select.projection.iter().enumerate() {
                            let hint = expected_types.get(idx).cloned().flatten();

                            match item {
                                sqlparser::ast::SelectItem::UnnamedExpr(expr)
                                | sqlparser::ast::SelectItem::ExprWithAlias { expr, .. } => {
                                    // Use the hint from the target table!
                                    traverse_expr(
                                        expr,
                                        &table_names,
                                        all_tables,
                                        &mut results,
                                        hint,
                                    )?;
                                }
                                _ => {}
                            }
                        }

                        // traverse the rest of the query (WHERE clauses, etc.)
                        // passing None as hint for the rest
                        if let Some(selection) = &select.selection {
                            traverse_expr(selection, &table_names, all_tables, &mut results, None)?;
                        }
                    }
                    _ => {
                        // For other cases (e.g. UNION), just traverse blindly
                        traverse_query(
                            source_query,
                            &table_names,
                            &mut working_tables,
                            &mut results,
                        )?;
                    }
                }
            }
        }
        _ => {}
    }
    Ok(results)
}
fn traverse_expr(
    expr: &Expr,
    table_names: &Vec<String>,
    all_tables: &mut HashMap<String, Vec<ColumnInfo>>,
    results: &mut Vec<Type>,
    parent_hint: Option<Type>, // The type inferred from the parent context
) -> Result<(), String> {
    match expr {
        Expr::Subquery(query) => {
            traverse_query(query, table_names, all_tables, results)?;
            Ok(())
        }

        Expr::Exists { subquery, .. } => {
            traverse_query(subquery, table_names, all_tables, results)?;
            Ok(())
        }

        Expr::InSubquery { expr, subquery, .. } => {
            traverse_expr(expr, table_names, all_tables, results, None)?;
            traverse_query(subquery, table_names, all_tables, results)?;
            Ok(())
        }
        Expr::Value(val) => {
            if let sqlparser::ast::Value::Placeholder(_) = val.value {
                let t = parent_hint
                    .ok_or("Could not infer type for placeholder (ambiguous context)")?;
                if t.base_type == BaseType::PlaceHolder || t.base_type == BaseType::Unknowns {
                    return Err("Ambiguous context: Unable to infer a concrete type. Try casting one of the operands.".to_string());
                }

                results.push(t);
            }
            Ok(())
        }

        Expr::Like {
            expr,
            pattern,
            escape_char,
            ..
        } => {
            traverse_expr(
                expr,
                table_names,
                all_tables,
                results,
                Some(Type {
                    base_type: BaseType::Text,
                    nullable: true,
                    contains_placeholder: false,
                }),
            )?;

            traverse_expr(
                pattern,
                table_names,
                all_tables,
                results,
                Some(Type {
                    base_type: BaseType::Text,
                    nullable: true,
                    contains_placeholder: false,
                }),
            )?;

            if let Some(escape) = escape_char {
                let escape_expr = Expr::Value(escape.clone().into());
                traverse_expr(
                    &escape_expr,
                    table_names,
                    all_tables,
                    results,
                    Some(Type {
                        base_type: BaseType::Text,
                        nullable: true,
                        contains_placeholder: false,
                    }),
                )?;
            }
            Ok(())
        }
        // In traverse_expr match block:
        Expr::IsNull(expr) | Expr::IsNotNull(expr) => {
            // We cannot infer type from "IS NULL", but we must verify the child doesn't contain a raw placeholder
            traverse_expr(expr, table_names, all_tables, results, None)?;
            Ok(())
        }

        Expr::IsTrue(expr)
        | Expr::IsFalse(expr)
        | Expr::IsNotTrue(expr)
        | Expr::IsNotFalse(expr) => {
            traverse_expr(
                expr,
                table_names,
                all_tables,
                results,
                Some(Type {
                    base_type: BaseType::Bool,
                    nullable: false,
                    contains_placeholder: false,
                }),
            )?;
            Ok(())
        }
        Expr::UnaryOp { expr, .. } => {
            traverse_expr(expr, table_names, all_tables, results, parent_hint)?;
            Ok(())
        }

        Expr::BinaryOp { left, right, op } => {
            // evaluates types of children purely to get context for the OTHER child
            let left_type = evaluate_expr_type(left, table_names, all_tables)?;
            let right_type = evaluate_expr_type(right, table_names, all_tables)?;

            let (left_hint, right_hint) = match op {
                BinaryOperator::Eq
                | BinaryOperator::Gt
                | BinaryOperator::Lt
                | BinaryOperator::GtEq
                | BinaryOperator::LtEq
                | BinaryOperator::NotEq => (Some(right_type), Some(left_type)),

                BinaryOperator::Plus
                | BinaryOperator::Minus
                | BinaryOperator::Multiply
                | BinaryOperator::Modulo
                | BinaryOperator::Divide => (Some(right_type), Some(left_type)),

                BinaryOperator::StringConcat => {
                    let text_type = Type {
                        base_type: BaseType::Text,
                        nullable: true,
                        contains_placeholder: false,
                    };
                    (Some(text_type.clone()), Some(text_type))
                }

                BinaryOperator::And | BinaryOperator::Or => {
                    let bool_type = Type {
                        base_type: BaseType::Bool,
                        nullable: false,
                        contains_placeholder: false,
                    };
                    (Some(bool_type.clone()), Some(bool_type))
                }

                BinaryOperator::BitwiseOr
                | BinaryOperator::BitwiseAnd
                | BinaryOperator::BitwiseXor => {
                    let int_type = Type {
                        base_type: BaseType::Integer,
                        nullable: true,
                        contains_placeholder: false,
                    };
                    (Some(int_type.clone()), Some(int_type))
                }

                _ => (None, None),
            };

            traverse_expr(left, table_names, all_tables, results, left_hint)?;
            traverse_expr(right, table_names, all_tables, results, right_hint)?;

            Ok(())
        }
        Expr::Nested(inner) => traverse_expr(inner, table_names, all_tables, results, parent_hint),

        Expr::InList { expr, list, .. } => {
            // First, check the Left Side (e.g. "id IN ...")
            let mut common_type = evaluate_expr_type(expr, table_names, all_tables)
                .ok()
                .filter(|t| {
                    t.base_type != BaseType::PlaceHolder && t.base_type != BaseType::Unknowns
                });

            // If Left Side gave nothing (it was '?'), check the List (e.g. "... IN (1, 2)")
            if common_type.is_none() {
                for item in list {
                    if let Ok(t) = evaluate_expr_type(item, table_names, all_tables)
                        && t.base_type != BaseType::PlaceHolder && t.base_type != BaseType::Unknowns
                        {
                            common_type = Some(t);
                            break;
                        }
                }
            }

            // 2. Traverse the Left Side with the Context
            // (If common_type is None here, this will trigger the "Ambiguous" error on the LHS ?)
            traverse_expr(expr, table_names, all_tables, results, common_type.clone())?;

            // 3. Traverse the List with the Context
            for item in list {
                traverse_expr(item, table_names, all_tables, results, common_type.clone())?;
            }

            Ok(())
        }
        Expr::Between {
            expr, low, high, ..
        } => {
            let e_type = evaluate_expr_type(expr, table_names, all_tables);
            let l_type = evaluate_expr_type(low, table_names, all_tables);
            let h_type = evaluate_expr_type(high, table_names, all_tables);

            // Determine context from the first available non-placeholder type
            let context = if let Ok(t) = &e_type
                && t.base_type != BaseType::PlaceHolder
            {
                Some(t.clone())
            } else if let Ok(t) = &l_type
                && t.base_type != BaseType::PlaceHolder
            {
                Some(t.clone())
            } else if let Ok(t) = &h_type
                && t.base_type != BaseType::PlaceHolder
            {
                Some(t.clone())
            } else {
                None // If all 3 are placeholders, we can't infer anything.
            };

            // Recursion strictly Left-to-Right
            traverse_expr(expr, table_names, all_tables, results, context.clone())?;
            traverse_expr(low, table_names, all_tables, results, context.clone())?;
            traverse_expr(high, table_names, all_tables, results, context)?;
            Ok(())
        }
        Expr::Cast {
            expr, data_type, ..
        } => {
            let target_type = match data_type {
                DataType::Int(_)
                | DataType::Integer(_)
                | DataType::TinyInt(_)
                | DataType::SmallInt(_)
                | DataType::MediumInt(_)
                | DataType::BigInt(_)
                | DataType::BigIntUnsigned(_)
                | DataType::Int2(_)
                | DataType::Int8(_) => Some(Type {
                    base_type: BaseType::Integer,
                    nullable: true,
                    contains_placeholder: false,
                }),

                DataType::Character(_)
                | DataType::Varchar(_)
                | DataType::CharVarying(_)
                | DataType::CharacterVarying(_)
                | DataType::Nvarchar(_)
                | DataType::Text
                | DataType::Clob(_) => Some(Type {
                    base_type: BaseType::Text,
                    nullable: true,
                    contains_placeholder: false,
                }),

                DataType::Real
                | DataType::Double(_)
                | DataType::DoublePrecision
                | DataType::Numeric(_)
                | DataType::Decimal(_)
                | DataType::Float(_) => Some(Type {
                    base_type: BaseType::Real,
                    nullable: true,
                    contains_placeholder: false,
                }),

                DataType::Boolean => Some(Type {
                    base_type: BaseType::Bool,
                    nullable: true,
                    contains_placeholder: false,
                }),

                _ => None,
            };

            traverse_expr(expr, table_names, all_tables, results, target_type)?;
            Ok(())
        }

        Expr::Case {
            operand,
            conditions,
            else_result,
            ..
        } => {
            // --- STEP 1: Determine the Return Type Hint ---
            // We look for the first non-placeholder type in THEN or ELSE clauses.
            // If found, that becomes the hint for all other result branches.
            let mut result_hint = parent_hint.clone();

            // A. Check 'ELSE' first (often simplest)
            if result_hint.is_none()
                && let Some(else_expr) = else_result
                && let Ok(t) = evaluate_expr_type(else_expr, table_names, all_tables)
                && t.base_type != BaseType::PlaceHolder
                && t.base_type != BaseType::Unknowns
            {
                result_hint = Some(t);
            }

            // B. Check 'THEN' clauses if we still don't know
            if result_hint.is_none() {
                for cond in conditions {
                    if let Ok(t) = evaluate_expr_type(&cond.result, table_names, all_tables)
                        && t.base_type != BaseType::PlaceHolder
                        && t.base_type != BaseType::Unknowns
                    {
                        result_hint = Some(t);
                        break;
                    }
                }
            }

            // --- STEP 2: Determine the Operand/Condition Hint ---
            let operand_hint = if let Some(op_expr) = operand {
                // Evaluate the operand (e.g., CASE x WHEN 1...) to hint the WHEN clauses
                traverse_expr(op_expr, table_names, all_tables, results, None)?;
                evaluate_expr_type(op_expr, table_names, all_tables).ok()
            } else {
                None
            };

            // --- STEP 3: Traverse ---
            for cond in conditions {
                // 1. Condition (WHEN ...)
                // If it's "CASE x WHEN ...", hint matches x.
                // If it's "CASE WHEN ...", hint is Boolean.
                let when_hint = operand_hint.clone().or(Some(Type {
                    base_type: BaseType::Bool,
                    nullable: false,
                    contains_placeholder: false,
                }));

                traverse_expr(&cond.condition, table_names, all_tables, results, when_hint)?;

                // 2. Result (THEN ...) -> Uses the result_hint we found in Step 1
                traverse_expr(
                    &cond.result,
                    table_names,
                    all_tables,
                    results,
                    result_hint.clone(),
                )?;
            }

            // 3. Else (ELSE ...) -> Uses the result_hint
            if let Some(else_expr) = else_result {
                traverse_expr(else_expr, table_names, all_tables, results, result_hint)?;
            }

            Ok(())
        }
        Expr::Function(func) => {
            let name = func.name.to_string().to_uppercase();

            let expects_text = matches!(
                name.as_str(),
                "LOWER"
                    | "UPPER"
                    | "LTRIM"
                    | "RTRIM"
                    | "TRIM"
                    | "REPLACE"
                    | "SUBSTR"
                    | "SUBSTRING"
                    | "CONCAT"
                    | "CONCAT_WS"
                    | "LENGTH"
                    | "OCTET_LENGTH"
                    | "INSTR"
                    | "GLOB"
                    | "LIKE"
                    | "DATE"
                    | "TIME"
                    | "DATETIME"
            );

            // Bucket 2: Functions that expect NUMERIC inputs (Real or Int)
            let expects_number = matches!(
                name.as_str(),
                "ABS"
                    | "ROUND"
                    | "CEIL"
                    | "FLOOR"
                    | "SIN"
                    | "COS"
                    | "TAN"
                    | "ASIN"
                    | "ACOS"
                    | "ATAN"
                    | "SQRT"
                    | "POW"
                    | "POWER"
                    | "LOG"
                    | "LOG10"
                    | "EXP"
                    | "DEGREES"
                    | "RADIANS"
                    | "SIGN"
                    | "MOD"
            );

            let is_polymorphic = matches!(
                name.as_str(),
                "COALESCE"
                    | "IFNULL"
                    | "MIN"
                    | "MAX"
                    | "SUM"
                    | "LEAD"
                    | "LAG"
                    | "NULLIF"
                    | "FIRST_VALUE"
                    | "LAST_VALUE"
            );

            let arg_hint = if expects_text {
                Some(Type {
                    base_type: BaseType::Text,
                    nullable: true,
                    contains_placeholder: false,
                })
            } else if expects_number {
                Some(Type {
                    base_type: BaseType::Real,
                    nullable: true,
                    contains_placeholder: false,
                })
            } else if is_polymorphic {
                // TODO check if this actually works
                let mut found_sibling = None;
                if let FunctionArguments::List(args_list) = &func.args {
                    for arg in &args_list.args {
                        if let FunctionArg::Unnamed(FunctionArgExpr::Expr(arg_expr)) = arg
                            && let Ok(t) = evaluate_expr_type(arg_expr, table_names, all_tables)
                            && t.base_type != BaseType::PlaceHolder
                        {
                            found_sibling = Some(t);
                            break;
                        }
                    }
                }
                // Use sibling type if found, otherwise fallback to what the parent expected of this function
                found_sibling.or(parent_hint)
            } else {
                // Unknown function or functions like COUNT(*) that don't care
                None
            };

            if let FunctionArguments::List(args_list) = &func.args {
                for arg in &args_list.args {
                    if let FunctionArg::Unnamed(FunctionArgExpr::Expr(arg_expr)) = arg {
                        traverse_expr(
                            arg_expr,
                            table_names,
                            all_tables,
                            results,
                            arg_hint.clone(),
                        )?;
                    }
                }
            }
            Ok(())
        }

        Expr::Ceil { expr, .. } | Expr::Floor { expr, .. } => {
            let hint = Some(Type {
                base_type: BaseType::Real,
                nullable: true,
                contains_placeholder: false,
            });
            traverse_expr(expr, table_names, all_tables, results, hint)?;
            Ok(())
        }

        Expr::Substring {
            expr,
            substring_from,
            substring_for,
            ..
        } => {
            traverse_expr(
                expr,
                table_names,
                all_tables,
                results,
                Some(Type {
                    base_type: BaseType::Text,
                    nullable: true,
                    contains_placeholder: false,
                }),
            )?;

            if let Some(from_expr) = substring_from {
                traverse_expr(
                    from_expr,
                    table_names,
                    all_tables,
                    results,
                    Some(Type {
                        base_type: BaseType::Integer,
                        nullable: true,
                        contains_placeholder: false,
                    }),
                )?;
            }

            if let Some(for_expr) = substring_for {
                traverse_expr(
                    for_expr,
                    table_names,
                    all_tables,
                    results,
                    Some(Type {
                        base_type: BaseType::Integer,
                        nullable: true,
                        contains_placeholder: false,
                    }),
                )?;
            }
            Ok(())
        }

        Expr::Trim {
            expr, trim_what, ..
        } => {
            let text_hint = Some(Type {
                base_type: BaseType::Text,
                nullable: true,
                contains_placeholder: false,
            });

            traverse_expr(expr, table_names, all_tables, results, text_hint.clone())?;

            if let Some(pattern_expr) = trim_what {
                traverse_expr(pattern_expr, table_names, all_tables, results, text_hint)?;
            }
            Ok(())
        }

        _ => Ok(()),
    }
}

// Add these functions at the bottom of the file

fn traverse_query(
    query: &sqlparser::ast::Query,
    table_names: &Vec<String>,
    all_tables: &mut HashMap<String, Vec<ColumnInfo>>,
    results: &mut Vec<Type>,
) -> Result<(), String> {
    // 1. Handle CTEs (WITH clause)
    if let Some(with) = &query.with {
        for cte in &with.cte_tables {
            // Traverse inside the CTE first
            // FIX: used 'table_names' instead of 'outer_scope_names'
            traverse_query(&cte.query, table_names, all_tables, results)?;

            // Infer Schema of the CTE
            let inferred_columns = infer_cte_columns(&cte.query, all_tables);
            let cte_name = cte.alias.name.value.clone();

            // Register CTE as if it were a real table
            all_tables.insert(cte_name, inferred_columns);
        }
    }

    // 2. Handle Body (Select, Union, Values, etc.)
    traverse_set_expr(&query.body, table_names, all_tables, results)?;

    // 3. Handle LIMIT / OFFSET parameters
    if let Some(limit_clause) = &query.limit_clause {
        let int_hint = Some(Type {
            base_type: BaseType::Integer,
            nullable: false,
            contains_placeholder: false,
        });

        if let sqlparser::ast::LimitClause::LimitOffset { limit, offset, .. } = limit_clause {
            if let Some(limit_expr) = limit {
                traverse_expr(
                    limit_expr,
                    table_names,
                    all_tables,
                    results,
                    int_hint.clone(),
                )?;
            }
            if let Some(offset_struct) = offset {
                traverse_expr(
                    &offset_struct.value,
                    table_names,
                    all_tables,
                    results,
                    int_hint,
                )?;
            }
        }
    }

    Ok(())
}
// binding_patterns.rs

fn infer_cte_columns(
    query: &sqlparser::ast::Query,
    all_tables: &HashMap<String, Vec<ColumnInfo>>,
) -> Vec<ColumnInfo> {
    if let sqlparser::ast::SetExpr::Select(select) = &*query.body {
        // 1. Build Scope
        let mut cte_scope = Vec::new();
        for table_with_joins in &select.from {
            if let sqlparser::ast::TableFactor::Table { name, .. } = &table_with_joins.relation {
                cte_scope.push(name.to_string());
            }
            for join in &table_with_joins.joins {
                if let sqlparser::ast::TableFactor::Table { name, .. } = &join.relation {
                    cte_scope.push(name.to_string());
                }
            }
        }

        let mut cols = Vec::new();
        for (i, item) in select.projection.iter().enumerate() {
            match item {
                // FIX: Handle SELECT *
                sqlparser::ast::SelectItem::Wildcard(_opt) => {
                    for table_name in &cte_scope {
                        if let Some(table_cols) = all_tables.get(table_name) {
                            cols.extend(table_cols.clone());
                        }
                    }
                }
                // FIX: Handle SELECT table.*
                sqlparser::ast::SelectItem::QualifiedWildcard(obj_name, _) => {
                    let table_name = obj_name.to_string();
                    if let Some(table_cols) = all_tables.get(&table_name) {
                        cols.extend(table_cols.clone());
                    }
                }
                // Normal columns
                sqlparser::ast::SelectItem::ExprWithAlias { alias, expr } => {
                    let deduced_type =
                        evaluate_expr_type(expr, &cte_scope, all_tables).unwrap_or(Type {
                            base_type: BaseType::Unknowns,
                            nullable: true,
                            contains_placeholder: false,
                        });
                    cols.push(ColumnInfo {
                        name: alias.value.clone(),
                        data_type: deduced_type,
                        check_constraint: None,
                    });
                }
                sqlparser::ast::SelectItem::UnnamedExpr(expr) => {
                    let col_name = match expr {
                        sqlparser::ast::Expr::Identifier(ident) => ident.value.clone(),
                        _ => format!("col_{}", i),
                    };
                    let deduced_type =
                        evaluate_expr_type(expr, &cte_scope, all_tables).unwrap_or(Type {
                            base_type: BaseType::Unknowns,
                            nullable: true,
                            contains_placeholder: false,
                        });
                    cols.push(ColumnInfo {
                        name: col_name,
                        data_type: deduced_type,
                        check_constraint: None,
                    });
                }
            }
        }
        return cols;
    }
    vec![]
}

#[allow(unused)]
fn traverse_set_expr(
    set_expr: &sqlparser::ast::SetExpr,
    outer_scope: &Vec<String>,
    all_tables: &mut HashMap<String, Vec<ColumnInfo>>,
    results: &mut Vec<Type>,
) -> Result<(), String> {
    match set_expr {
        sqlparser::ast::SetExpr::Select(select) => {
            let mut local_scope_tables = Vec::new();

            // Helper closure to register aliases
            let mut register_table = |relation: &sqlparser::ast::TableFactor| {
                if let sqlparser::ast::TableFactor::Table { name, alias, .. } = relation {
                    let real_name = name.to_string();
                    let effective_name = if let Some(a) = alias {
                        let alias_name = a.name.value.clone();
                        if let Some(cols) = all_tables.get(&real_name).cloned() {
                            all_tables.insert(alias_name.clone(), cols);
                        }
                        alias_name
                    } else {
                        real_name
                    };
                    local_scope_tables.push(effective_name);
                }
            };

            for table_with_joins in &select.from {
                register_table(&table_with_joins.relation);
                for join in &table_with_joins.joins {
                    register_table(&join.relation);
                }
            }

            // 1. Collect Projections (but do NOT register them yet)
            let mut projected_aliases = Vec::new();
            for item in &select.projection {
                match item {
                    sqlparser::ast::SelectItem::UnnamedExpr(expr) => {
                        traverse_expr(expr, &local_scope_tables, all_tables, results, None)?;
                    }
                    sqlparser::ast::SelectItem::ExprWithAlias { expr, alias } => {
                        traverse_expr(expr, &local_scope_tables, all_tables, results, None)?;

                        let derived_type =
                            evaluate_expr_type(expr, &local_scope_tables, all_tables).unwrap_or(
                                Type {
                                    base_type: BaseType::Unknowns,
                                    nullable: true,
                                    contains_placeholder: false,
                                },
                            );

                        projected_aliases.push(ColumnInfo {
                            name: alias.value.clone(),
                            data_type: derived_type,
                            check_constraint: None,
                        });
                    }
                    _ => {}
                }
            }

            // 2. Process Joins / ON clauses (Aliases not visible here)
            for table_with_joins in &select.from {
                for join in &table_with_joins.joins {
                    let constraint = match &join.join_operator {
                        sqlparser::ast::JoinOperator::Inner(c)
                        | sqlparser::ast::JoinOperator::Left(c)
                        | sqlparser::ast::JoinOperator::LeftOuter(c)
                        | sqlparser::ast::JoinOperator::Right(c)
                        | sqlparser::ast::JoinOperator::RightOuter(c)
                        | sqlparser::ast::JoinOperator::FullOuter(c)
                        | sqlparser::ast::JoinOperator::Join(c)
                        | sqlparser::ast::JoinOperator::CrossJoin(c) => Some(c),
                        _ => None,
                    };

                    if let Some(sqlparser::ast::JoinConstraint::On(expr)) = constraint {
                        traverse_expr(expr, &local_scope_tables, all_tables, results, None)?;
                    }
                }
            }

            // 3. Process WHERE (Aliases NOT visible here)
            // This fixes the shadowing issue. It will only see the real table column.
            if let Some(selection) = &select.selection {
                traverse_expr(selection, &local_scope_tables, all_tables, results, None)?;
            }

            // 4. NOW Inject Aliases (Visible for HAVING, GROUP BY, etc.)
            if !projected_aliases.is_empty() {
                let virtual_alias_table = "$_current_scope_aliases_$".to_string();
                all_tables.insert(virtual_alias_table.clone(), projected_aliases);
                local_scope_tables.push(virtual_alias_table);
            }

            // 5. Process HAVING (Aliases ARE visible here)
            if let Some(having) = &select.having {
                traverse_expr(having, &local_scope_tables, all_tables, results, None)?;
            }
        }

        sqlparser::ast::SetExpr::SetOperation { left, right, .. } => {
            traverse_set_expr(left, outer_scope, all_tables, results)?;
            traverse_set_expr(right, outer_scope, all_tables, results)?;
        }

        sqlparser::ast::SetExpr::Query(q) => {
            traverse_query(q, outer_scope, all_tables, results)?;
        }

        sqlparser::ast::SetExpr::Values(values) => {
            for row in &values.rows {
                for expr in row {
                    traverse_expr(expr, outer_scope, all_tables, results, None)?;
                }
            }
        }
        _ => {}
    }
    Ok(())
}
