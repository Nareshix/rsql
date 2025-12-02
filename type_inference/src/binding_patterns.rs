use std::{collections::HashMap, ops::ControlFlow};

use sqlparser::{
    ast::{
        Expr, Visit, Visitor,
    },
    dialect::SQLiteDialect,
    parser::Parser,
};

use crate::expr::{BaseType, evaluate_expr_type};
use crate::{
    expr::Type,
    table::{ColumnInfo, get_table_names},
};

// #[derive(Default)]
#[derive(Debug)]
struct V<'a> {
    types: Vec<Type>,
    table_names_from_select: &'a Vec<String>,
    all_tables: &'a HashMap<String, Vec<ColumnInfo>>,
}

trait IntoControlFlow<T> {
    fn into_cf(self) -> ControlFlow<String, T>;
}

impl<T> IntoControlFlow<T> for Result<T, String> {
    fn into_cf(self) -> ControlFlow<String, T> {
        match self {
            Ok(v) => ControlFlow::Continue(v),
            Err(e) => ControlFlow::Break(e),
        }
    }
}
impl Visitor for V<'_> {
    type Break = String;

    // it is guranteed for ? placeholder to always be used in an expression
    // and never standalone. The only time it is used standalone is in the
    // rare usage of SELECT ?,?,?... which will be handled by select_pattern.rs
    fn post_visit_expr(&mut self, expr: &Expr) -> ControlFlow<Self::Break> {
        match expr {
            Expr::BinaryOp { left, right, .. } => {
                let lhs_expr_type =
                    evaluate_expr_type(left, self.table_names_from_select, self.all_tables)
                        .into_cf()?;
                let rhs_expr_type =
                    evaluate_expr_type(right, self.table_names_from_select, self.all_tables)
                        .into_cf()?;

                // it guaranteed for either LHS or RHS to have 1 ?.
                // This is because if there is 2, it will be return as an error which is taken care of above
                if lhs_expr_type.base_type == BaseType::PlaceHolder {
                    self.types.push(rhs_expr_type);
                } else if rhs_expr_type.base_type == BaseType::PlaceHolder {
                    self.types.push(lhs_expr_type);
                }
            }
            Expr::InList { expr, list, .. } => {
                let lhs = evaluate_expr_type(expr, self.table_names_from_select, self.all_tables)
                    .into_cf()?;

                // Case: ? IN (1, 2)
                if lhs.base_type == BaseType::PlaceHolder
                    && let Some(first) = list.first() {
                        let first_type = evaluate_expr_type(
                            first,
                            self.table_names_from_select,
                            self.all_tables,
                        )
                        .into_cf()?;
                        self.types.push(first_type);
                    }

                // Case: id IN (?, ?)
                for item in list {
                    let item_type =
                        evaluate_expr_type(item, self.table_names_from_select, self.all_tables)
                            .into_cf()?;
                    if item_type.base_type == BaseType::PlaceHolder {
                        self.types.push(lhs.clone());
                    }
                }
            }
            Expr::Between {
                expr, low, high, ..
            } => {
                let middle =
                    evaluate_expr_type(expr, self.table_names_from_select, self.all_tables)
                        .into_cf()?;
                let low_t = evaluate_expr_type(low, self.table_names_from_select, self.all_tables)
                    .into_cf()?;
                let high_t =
                    evaluate_expr_type(high, self.table_names_from_select, self.all_tables)
                        .into_cf()?;

                // Case: ? BETWEEN 10 AND 20
                if middle.base_type == BaseType::PlaceHolder {
                    self.types.push(low_t.clone()); // Infer from low
                }

                // Case: age BETWEEN ? AND ?
                if low_t.base_type == BaseType::PlaceHolder {
                    self.types.push(middle.clone());
                }
                if high_t.base_type == BaseType::PlaceHolder {
                    self.types.push(middle.clone());
                }
            }
            Expr::Like { expr, pattern, .. } => {
                let lhs = evaluate_expr_type(expr, self.table_names_from_select, self.all_tables)
                    .into_cf()?;
                let rhs =
                    evaluate_expr_type(pattern, self.table_names_from_select, self.all_tables)
                        .into_cf()?;

                if lhs.base_type == BaseType::PlaceHolder {
                    self.types.push(rhs);
                }
                else if rhs.base_type == BaseType::PlaceHolder {
                    self.types.push(lhs);
                }
            }
            _ => {}
        }
        ControlFlow::Continue(())
    }
}

#[allow(unused)]
pub fn get_type_of_binding_parameters(
    sql: &str,
    all_tables: &HashMap<String, Vec<ColumnInfo>>,
) -> Result<Vec<Type>, String> {
    let statement = &Parser::parse_sql(&SQLiteDialect {}, sql).unwrap()[0];
    let table_names_from_select = &get_table_names(sql);
    let mut types = Vec::new();

    let mut visitor = V {
        all_tables,
        table_names_from_select,
        types,
    };

    match statement.visit(&mut visitor) {
        ControlFlow::Break(err_msg) => Err(err_msg),
        ControlFlow::Continue(_) => Ok(visitor.types),
    }

    // LIMIT and OFFSET
    // let check_placeholder = |expr: &Expr| {
    //     if matches!(
    //         expr,
    //         Expr::Value(ValueWithSpan {
    //             value: Value::Placeholder(_),
    //             ..
    //         })
    //     ) {
    //         Ok(Type {
    //             base_type: BaseType::Integer,
    //             nullable: false, //dont care wht this is
    //             contains_placeholder: true,
    //         })
    //     } else {
    //         Err("internal error? something went wrong. cant analyse LIMIT or OFFSET".to_string())
    //     }
    // };

    // if let Statement::Query(query) = statement
    //     && let Some(LimitClause::LimitOffset { limit, offset, .. }) = &query.limit_clause
    // {
    //     // LIMIT
    //     if let Some(limit_expr) = limit {
    //         let x = check_placeholder(limit_expr);
    //         types.push(x);
    //     }

    //     // OFFSET
    //     if let Some(offset_struct) = offset {
    //         let x = check_placeholder(&offset_struct.value);
    //         types.push(x);
    //     }
    // }
}
