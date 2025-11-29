use std::{collections::HashMap, ops::ControlFlow};

use sqlparser::{
    ast::{Expr, LimitClause, Statement, Value, ValueWithSpan, visit_expressions},
    dialect::SQLiteDialect,
    parser::Parser,
};

use crate::expr::{BaseType, evaluate_expr_type};
use crate::{
    expr::Type,
    table::{ColumnInfo, get_table_names},
};

pub fn get_type_of_binding_parameters(
    sql: &str,
    all_tables: &HashMap<String, Vec<ColumnInfo>>,
) -> Vec<Result<Type, String>> {
    let statements = Parser::parse_sql(&SQLiteDialect {}, sql).unwrap();
    let table_names_from_select = get_table_names(sql);
    let mut types = Vec::new();

    // LHS op RHS (including Lists and exists, TODO exist)
    let _ = visit_expressions(&statements, |expr| {
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
                if let Expr::Value(ValueWithSpan { value, .. }) = &list[0]
                    && let Value::Placeholder(_) = value
                {
                    types.push(evaluate_expr_type(
                        expr,
                        &table_names_from_select,
                        all_tables,
                    ));
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
                if low_is_ph || high_is_ph {
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

    types

    // if let ControlFlow::Break(result) = visit_exp {
    //     return types;
    // } else {
    //     return  types;
    // }

    // // LIMIT and OFFSET
    // let check_placeholder = |expr: &Expr| {
    //     if matches!(
    //         expr,
    //         Expr::Value(ValueWithSpan {
    //             value: Value::Placeholder(_),
    //             ..
    //         })
    //     ) {
    //         println!("int");
    //         Ok(Type {
    //             base_type: BaseType::Integer,
    //             nullable: false, //dont care wht this is
    //         })
    //     } else {
    //         Err("internal error? something went wrong. cant analyse LIMIT or OFFSET".to_string())
    //     }
    // };

    // for statement in statements {
    //     if let Statement::Query(query) = statement
    //         && let Some(LimitClause::LimitOffset { limit, offset, .. }) = query.limit_clause
    //     {
    //         // LIMIT
    //         if let Some(limit_expr) = limit {
    //             let x = check_placeholder(&limit_expr);
    //         }

    //         // OFFSET
    //         if let Some(offset_struct) = offset {
    //             let x = check_placeholder(&offset_struct.value);
    //         }
    //     }
    // }
    // x
}
