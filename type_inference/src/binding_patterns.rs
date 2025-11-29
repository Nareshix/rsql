use core::panic;
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
) -> Result<Type, String> {
    let statements = Parser::parse_sql(&SQLiteDialect {}, sql).unwrap();

    let table_names_from_select = get_table_names(sql);
    let visit_exp = visit_expressions(&statements, |expr| {
        match expr {
            Expr::BinaryOp { left, right, .. } => {
                if let Expr::Value(ValueWithSpan { value, .. }) = &**right
                    && let Value::Placeholder(_) = value
                {
                    return ControlFlow::Break(evaluate_expr_type(
                        left,
                        &table_names_from_select,
                        all_tables,
                    ));
                }
            }
            Expr::InList { expr, list, .. } => {
                // assume that all the InList contains ?
                if let Expr::Value(ValueWithSpan { value, .. }) = &list[0]
                    && let Value::Placeholder(_) = value
                {
                    return ControlFlow::Break(evaluate_expr_type(
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
                    return ControlFlow::Break(evaluate_expr_type(
                        expr,
                        &table_names_from_select,
                        all_tables,
                    ));
                }
            }
            _ => {}
        }

        ControlFlow::Continue(())
    });

    match visit_exp {
        ControlFlow::Break(result) => result,
        ControlFlow::Continue(_) => Err("No binding parameters found to analyze".to_string()),
    }
}
