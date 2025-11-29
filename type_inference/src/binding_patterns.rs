use std::{collections::HashMap, ops::ControlFlow};

use sqlparser::{
    ast::{AssignmentTarget, Expr, LimitClause, ObjectName, ObjectNamePart, Statement, Value, ValueWithSpan, visit_expressions},
    dialect::SQLiteDialect,
    parser::Parser,
};

use crate::expr::{BaseType, evaluate_expr_type};
use crate::{
    expr::Type,
    table::{ColumnInfo, get_table_names},
};

#[allow(unused)]
pub fn get_type_of_binding_parameters(
    sql: &str,
    all_tables: &HashMap<String, Vec<ColumnInfo>>,
) -> Vec<Result<Type, String>> {
    let statement = &Parser::parse_sql(&SQLiteDialect {}, sql).unwrap()[0];
    let table_names_from_select = get_table_names(sql);
    let mut types = Vec::new();

    //Checks if it is Update and only one binding parameter SET = ?. any other expression in RHS would be resovled below
    if let Statement::Update { assignments, .. } = &statement {
        for assignment in assignments {
            let evaluated_expr_type = evaluate_expr_type(&assignment.value, &table_names_from_select, all_tables);
            if let Ok(x) = evaluated_expr_type
                && x.contains_placeholder{
                    let assignment_target = &assignment.target;
            if let AssignmentTarget::ColumnName(c) = assignment_target {
                for col in &c.0 {
                    // Rename 'expr' to 'ident' here for clarity. It is of type &Ident
                    if let ObjectNamePart::Identifier(ident) = col {

                        // Wrap the Ident into an Expr::Identifier
                        let expr_wrapper = Expr::Identifier(ident.clone());

                        types.push(evaluate_expr_type(
                            &expr_wrapper, // Pass the Expr, not the Ident
                            &table_names_from_select,
                            all_tables
                        ));
                    }
                }
            }

                    // types.push(evaluate_expr_type(&assignment.target, &table_names_from_select, all_tables));
                }

        }
    }

    // LHS op RHS (including Lists and exists, TODO exist)
    let _ = visit_expressions(statement, |expr| {
        match expr {
            Expr::BinaryOp { left, right, .. } => {
                if let Expr::Value(ValueWithSpan { value, .. }) = &**right
                    && let Value::Placeholder(_) = value
                {
                    types.push(evaluate_expr_type(
                        left,
                        &table_names_from_select,
                        all_tables,
                    ));
                }
            }
            Expr::InList { expr, list, .. } => {
                // assumed all are ? in the list
                if let Expr::Value(ValueWithSpan { value, .. }) = &list[0]
                    && let Value::Placeholder(_) = value
                {
                    for _ in 0..list.len() {
                        types.push(evaluate_expr_type(
                            expr,
                            &table_names_from_select,
                            all_tables,
                        ));
                    }
                }
            }
            Expr::Between {
                expr, low, high, ..
            } => {
                let low_is_ph = matches!(
                    &**low,
                    Expr::Value(ValueWithSpan {
                        value: Value::Placeholder(_),
                        ..
                    })
                );
                let high_is_ph = matches!(
                    &**high,
                    Expr::Value(ValueWithSpan {
                        value: Value::Placeholder(_),
                        ..
                    })
                );

                let mut num = 0;

                if low_is_ph {
                    num += 1
                }
                if high_is_ph {
                    num += 1
                }
                for _ in 0..num {
                    types.push(evaluate_expr_type(
                        expr,
                        &table_names_from_select,
                        all_tables,
                    ));
                }
            }

            // LIKE
            Expr::Like { expr, pattern, .. } => {
                if let Expr::Value(ValueWithSpan { value, .. }) = &**pattern
                    && let Value::Placeholder(_) = value
                {
                    types.push(evaluate_expr_type(
                        expr,
                        &table_names_from_select,
                        all_tables,
                    ));
                }
            }
            _ => {}
        }
        ControlFlow::<()>::Continue(())
    });

    // LIMIT and OFFSET
    let check_placeholder = |expr: &Expr| {
        if matches!(
            expr,
            Expr::Value(ValueWithSpan {
                value: Value::Placeholder(_),
                ..
            })
        ) {
            Ok(Type {
                base_type: BaseType::Integer,
                nullable: false, //dont care wht this is
                contains_placeholder: true
            })
        } else {
            Err("internal error? something went wrong. cant analyse LIMIT or OFFSET".to_string())
        }
    };

    if let Statement::Query(query) = statement
        && let Some(LimitClause::LimitOffset { limit, offset, .. }) = &query.limit_clause
    {
        // LIMIT
        if let Some(limit_expr) = limit {
            let x = check_placeholder(limit_expr);
            types.push(x);
        }

        // OFFSET
        if let Some(offset_struct) = offset {
            let x = check_placeholder(&offset_struct.value);
            types.push(x);
        }
    }
    types
}
