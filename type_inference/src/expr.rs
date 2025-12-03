use std::collections::HashMap;

use sqlparser::ast::{
    BinaryOperator, DataType, Expr, FunctionArg, FunctionArgExpr, FunctionArguments, Value,
};

use crate::table::ColumnInfo;
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BaseType {
    Integer,
    Real,
    Bool,
    Text,
    Null,
    Unknowns,
    PlaceHolder,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Type {
    pub base_type: BaseType,
    pub nullable: bool,
    pub contains_placeholder: bool,
}

/// https://docs.rs/sqlparser/latest/sqlparser/ast/enum.Expr.html, version 0.59.0
pub fn evaluate_expr_type(
    expr: &Expr,
    table_names_from_select: &Vec<String>,
    all_tables: &HashMap<String, Vec<ColumnInfo>>,
) -> Result<Type, String> {
    match expr {
        Expr::Identifier(ident) => {
            let col_name = &ident.value;
            let mut found_type: Option<Type> = None;

            for table_name in table_names_from_select {
                if let Some(column_infos) = all_tables.get(table_name) {
                    for column_info in column_infos {
                        if column_info.name == *col_name {
                            // Check if we already found a match previously
                            if found_type.is_some() {
                                return Err(format!(
                                    "Ambiguous column name '{}': It exists in multiple tables {:?}",
                                    col_name, table_names_from_select
                                ));
                            }
                            found_type = Some(column_info.data_type.clone());
                        }
                    }
                }
            }

            // Return the found type, or error if None found
            found_type.ok_or_else(|| {
                format!(
                    "Column '{}' not found in tables {:?}",
                    col_name, table_names_from_select
                )
            })
        }

        Expr::CompoundIdentifier(idents) => {
            // We expect 2 parts, e.g., "table.column"
            let table_name = &idents[0].value;
            let col_name = &idents[1].value;

            let column_infos = &all_tables[table_name];
            for column_info in column_infos {
                if column_info.name == *col_name {
                    return Ok(column_info.data_type.clone());
                }
            }
            Err(format!(
                "Column '{}' not found in table '{}'",
                col_name, table_name
            ))
        }

        // Expr::CompoundFieldAccess {..}
        // Expr::JsonAccess {..}
        // Expr::TypedString(_) -- TODO
        // Expr::GroupingSets() TODO
        // Expr::Cube() TODO
        // Expr::Rollup() TODO
        // Expr::Tuple(_) TODO
        // Expr::Struct { _ }
        // Expr::Array() sqlite dont have array type
        // Expr::Wildcard() --handled in select_pattern.rs
        // Expr::QualifiedWildcard(, ) --handled in select_pattern.rs
        // Expr::OuterJoin() --handled in creation of table
        // Expr::Prior() TODO
        // Expr::MemberOf() json specifc TODO
        // Compound
        Expr::Subquery(query) => {
            // Check the first item in the subquery's projection
            if let sqlparser::ast::SetExpr::Select(select) = &*query.body
                && let Some(item) = select.projection.first()
            {
                match item {
                    sqlparser::ast::SelectItem::UnnamedExpr(expr)
                    | sqlparser::ast::SelectItem::ExprWithAlias { expr, .. } => {
                        return evaluate_expr_type(expr, table_names_from_select, all_tables);
                    }
                    _ => {}
                }
            }
            Ok(Type {
                base_type: BaseType::Unknowns,
                nullable: true,
                contains_placeholder: false,
            })
        }

        // Raw Values e.g. SELECT 1 or SELECT "hello"
        Expr::Value(val) => {
            // identifies whether its a float or int
            let value = &val.value;
            match value {
                Value::Number(num, _) => {
                    if num.contains(".") {
                        return Ok(Type {
                            base_type: BaseType::Real,
                            nullable: false,
                            contains_placeholder: false,
                        });
                    }
                    Ok(Type {
                        base_type: BaseType::Integer,
                        nullable: false,
                        contains_placeholder: false,
                    })
                }
                Value::SingleQuotedString(_) => Ok(Type {
                    base_type: BaseType::Text,
                    nullable: false,
                    contains_placeholder: false,
                }),
                Value::DoubleQuotedString(_) => Ok(Type {
                    base_type: BaseType::Text,
                    nullable: false,
                    contains_placeholder: false,
                }),
                Value::Boolean(_) => Ok(Type {
                    base_type: BaseType::Bool,
                    nullable: false,
                    contains_placeholder: false,
                }),
                Value::Null => Ok(Type {
                    base_type: BaseType::Null,
                    nullable: true,
                    contains_placeholder: false,
                }),
                Value::Placeholder(_) => Ok(Type {
                    base_type: BaseType::PlaceHolder,
                    nullable: true,
                    contains_placeholder: true,
                }),

                _ => Err(format!(
                    "{value} is an invalid type. Make sure it is TEXT, INTEGER or REAL"
                )),
            }
        }

        // these always return a bool, regardless of input.
        Expr::IsNull(_)
        | Expr::IsNotNull(_)
        | Expr::IsTrue(_)
        | Expr::IsFalse(_)
        | Expr::IsNotFalse(_)
        | Expr::IsNotTrue(_)
        | Expr::IsDistinctFrom(..)
        | Expr::IsNotDistinctFrom(..)
        | Expr::Exists { .. } => Ok(Type {
            base_type: BaseType::Bool,
            nullable: false,
            contains_placeholder: false,
        }), // TODO placeholder
        // TODO Exists can be null, but usually they are subquerys

        // SELECT... WHERE id in (?,?,...)
        Expr::InList { expr, list, .. } => {
            let lhs = evaluate_expr_type(expr, table_names_from_select, all_tables)?;

            let mut contains_placeholder = lhs.contains_placeholder;
            let mut nullable = lhs.nullable;

            for item in list {
                let item_type = evaluate_expr_type(item, table_names_from_select, all_tables)?;

                contains_placeholder = contains_placeholder || item_type.contains_placeholder;
                nullable = nullable || item_type.nullable;
            }

            Ok(Type {
                base_type: BaseType::Bool,
                nullable,
                contains_placeholder,
            })
        }

        Expr::Between {
            expr, low, high, ..
        } => {
            let expr_type = evaluate_expr_type(expr, table_names_from_select, all_tables)?;
            let lhs = evaluate_expr_type(low, table_names_from_select, all_tables)?;
            let rhs = evaluate_expr_type(high, table_names_from_select, all_tables)?;

            let nullable = expr_type.nullable || lhs.nullable || rhs.nullable;
            let contains_placeholder = expr_type.contains_placeholder
                || lhs.contains_placeholder
                || rhs.contains_placeholder;

            Ok(Type {
                base_type: BaseType::Bool,
                nullable,
                contains_placeholder,
            })
        }

        Expr::Like {
            expr,
            pattern,
            escape_char,
            ..
        } => {
            let lhs = evaluate_expr_type(expr, table_names_from_select, all_tables)?;
            let rhs = evaluate_expr_type(pattern, table_names_from_select, all_tables)?;

            let mut contains_placeholder = lhs.contains_placeholder || rhs.contains_placeholder;
            let mut nullable = lhs.nullable || rhs.nullable;

            // Handle escape char (e.g. LIKE '100\%' ESCAPE '\')
            if let Some(escape_expr) = escape_char {
                // escape_expr is Value, and we need to wrap it in an Expr.
                let escape_expr = Expr::value(escape_expr.clone());

                let escape_type =
                    evaluate_expr_type(&escape_expr, table_names_from_select, all_tables)?;
                if escape_type.contains_placeholder {
                    contains_placeholder = true;
                }
                if escape_type.nullable {
                    nullable = true;
                }
            }

            Ok(Type {
                base_type: BaseType::Bool,
                nullable,
                contains_placeholder,
            })
        }

        Expr::InSubquery { expr, .. } => {
            let _lhs = evaluate_expr_type(expr, table_names_from_select, all_tables)?;

            let lhs_contains_placeholder = _lhs.contains_placeholder;
            let lhs_nullable = _lhs.nullable;

            Ok(Type {
                base_type: BaseType::Bool,
                nullable: lhs_nullable,
                contains_placeholder: lhs_contains_placeholder,
            })
        }
        Expr::BinaryOp { left, op, right } => {
            let left_type = evaluate_expr_type(left, table_names_from_select, all_tables)?;
            let right_type = evaluate_expr_type(right, table_names_from_select, all_tables)?;
            let propagates_placeholder =
                left_type.contains_placeholder || right_type.contains_placeholder;

            match op {
                // Comparisons always return Bool
                BinaryOperator::Eq
                | BinaryOperator::NotEq
                | BinaryOperator::Gt
                | BinaryOperator::Lt
                | BinaryOperator::GtEq
                | BinaryOperator::LtEq
                | BinaryOperator::And
                | BinaryOperator::Or => {
                    let are_comparable = match (&left_type.base_type, &right_type.base_type) {
                        (BaseType::PlaceHolder, BaseType::PlaceHolder) => {
                            return Err(format!(
                                "Unable to infer type '? {} ?'. Try Casting either one, or both of them",
                                *op
                            ));
                        }

                        (BaseType::PlaceHolder, _) | (_, BaseType::PlaceHolder) => true,

                        (BaseType::Null, _) | (_, BaseType::Null) => true,
                        (BaseType::Integer, BaseType::Integer) => true,
                        (BaseType::Real, BaseType::Real) => true,
                        (BaseType::Text, BaseType::Text) => true,
                        (BaseType::Bool, BaseType::Bool) => true,
                        (BaseType::Integer, BaseType::Real)
                        | (BaseType::Real, BaseType::Integer) => true,

                        _ => false,
                    };

                    if !are_comparable {
                        return Err(format!(
                            "Cannot compare types '{:?}' and '{:?}'",
                            left_type.base_type, right_type.base_type
                        ));
                    }

                    Ok(Type {
                        base_type: BaseType::Bool,
                        nullable: left_type.nullable || right_type.nullable,
                        contains_placeholder: propagates_placeholder,
                    })
                }

                BinaryOperator::Plus
                | BinaryOperator::Minus
                | BinaryOperator::Multiply
                | BinaryOperator::Modulo
                | BinaryOperator::Divide => {
                    let resolved_base_type = match (&left_type.base_type, &right_type.base_type) {
                        // If one side is a raw placeholder (? + 1), take the type of the non raw placeholder.
                        (BaseType::PlaceHolder, t) | (t, BaseType::PlaceHolder) => {
                            if *t == BaseType::PlaceHolder {
                                // ? + ?
                                return Err(format!(
                                    "Unable to infer type '? {} ?'. Try Casting either one, or both of them",
                                    *op
                                ));
                            } else {
                                *t
                            }
                        }

                        // 1 + NULL -> Result is NULL, but type should be Integer (nullable).
                        (BaseType::Null, t) | (t, BaseType::Null) => {
                            if *t == BaseType::Null {
                                BaseType::Null
                            } else {
                                *t
                            }
                        }

                        (BaseType::Integer, BaseType::Integer) => BaseType::Integer,
                        (BaseType::Real, BaseType::Real)
                        | (BaseType::Integer, BaseType::Real)
                        | (BaseType::Real, BaseType::Integer) => BaseType::Real,

                        (BaseType::Unknowns, _) | (_, BaseType::Unknowns) => BaseType::Unknowns,

                        _ => {
                            return Err(format!(
                                "Cannot apply math operator to types '{:?}' and '{:?}'",
                                left_type.base_type, right_type.base_type
                            ));
                        }
                    };

                    let is_op_nullable =
                        *op == BinaryOperator::Divide || *op == BinaryOperator::Modulo;
                    let resolved_nullable =
                        left_type.nullable || right_type.nullable || is_op_nullable;

                    Ok(Type {
                        base_type: resolved_base_type,
                        nullable: resolved_nullable,
                        contains_placeholder: propagates_placeholder,
                    })
                }

                // String concat always returns string
                BinaryOperator::StringConcat => Ok(Type {
                    base_type: BaseType::Text,
                    nullable: left_type.nullable || right_type.nullable,
                    contains_placeholder: propagates_placeholder,
                }),

                BinaryOperator::BitwiseOr
                | BinaryOperator::BitwiseAnd
                | BinaryOperator::BitwiseXor => Ok(Type {
                    base_type: BaseType::Integer,
                    nullable: true,
                    contains_placeholder: false,
                }),

                // TODO REGEXP. it is sqlite specific
                _ => Err(format!("invalid {expr}")),
            }
        }

        Expr::UnaryOp { op, expr } => {
            // op is one the 3, (+, -, NOT)
            match op {
                // +, -
                sqlparser::ast::UnaryOperator::Plus | sqlparser::ast::UnaryOperator::Minus => {
                    evaluate_expr_type(expr, table_names_from_select, all_tables)
                }

                sqlparser::ast::UnaryOperator::Not => {
                    evaluate_expr_type(expr, table_names_from_select, all_tables)
                }

                _ => Err(format!("invalid {expr}")),
            }
        }

        // Nested expression e.g. (foo > bar) or (1)
        Expr::Nested(inner_expr) => {
            evaluate_expr_type(inner_expr, table_names_from_select, all_tables)
        }

        // Expr::Convert {..}
        Expr::Cast { data_type, .. } => {
            // http://www.sqlite.org/datatype3.html#affinity_name_examples
            match data_type {
                DataType::Int(_)
                | DataType::Integer(_)
                | DataType::TinyInt(_)
                | DataType::SmallInt(_)
                | DataType::MediumInt(_)
                | DataType::BigInt(_)
                | DataType::BigIntUnsigned(_)
                | DataType::Int2(_)
                | DataType::Int8(_) => Ok(Type { base_type: BaseType::Integer, nullable: true, contains_placeholder: false }),

                DataType::Character(_)
                | DataType::Varchar(_)
                | DataType::CharVarying(_)
                | DataType::CharacterVarying(_)
                // sqlparser does not have  NCHAR(55)
                // sqlparser does not have  NATIVE CHARACTER(70)
                | DataType::Nvarchar(_)
                | DataType::Text
                | DataType::Clob(_) => Ok(Type { base_type: BaseType::Text, nullable: true, contains_placeholder: false }),

                // TODO
                // DataType::Blob(_) =>

                DataType::Real
                | DataType::Double(_)
                | DataType::DoublePrecision
                | DataType::Numeric(_) //undocumented but works
                | DataType::Decimal(_) //undocumented but works
                | DataType::Float(_) => Ok(Type { base_type: BaseType::Real, nullable: true, contains_placeholder: false }),


                // TODO Numeric

                _ => Err(format!("invalid data type {}", data_type))

            }
        }

        // some expressions have their own enum which couldve been inside the Function enum but isnt.
        // we have to handle those cases seperately. https://docs.rs/sqlparser/latest/sqlparser/ast/enum.Expr.html, sqlparser-0.59

        // ? category
        // Expr::Overlay { .. }
        // Expr::Collate { } TODO

        // Datetime
        // Expr::AtTimeZone {..}  TODO

        // Expr::Extract {.. }   TODO use strftime() instead
        // Expr::Position { .. } TODO use instr

        // Functions
        Expr::Function(func) => {
            let name = func.name.to_string().to_uppercase();

            let mut input_type = Type {
                base_type: BaseType::Unknowns,
                nullable: false,
                contains_placeholder: false,
            };
            let mut any_arg_nullable = false;
            let mut all_args_nullable = true; // track for COALESCE and ifnull

            if let FunctionArguments::List(list) = &func.args {
                for arg in &list.args {
                    if let FunctionArg::Unnamed(FunctionArgExpr::Expr(expr)) = arg {
                        let arg_type =
                            evaluate_expr_type(expr, table_names_from_select, all_tables)?;

                        if arg_type.nullable {
                            any_arg_nullable = true;
                        } else {
                            all_args_nullable = false;
                        }

                        // If we already have a type, and the new arg is different (and neither are null), it's Unknown.
                        if input_type.base_type == BaseType::Null
                            || input_type.base_type == BaseType::Unknowns
                        {
                            // Initialize type from first non-null arg
                            input_type = arg_type;
                        } else if arg_type.base_type != BaseType::Null
                            && input_type.base_type != arg_type.base_type
                        {
                            // Allow Int -> Real promotion
                            if (input_type.base_type == BaseType::Integer
                                && arg_type.base_type == BaseType::Real)
                                || (input_type.base_type == BaseType::Real
                                    && arg_type.base_type == BaseType::Integer)
                            {
                                input_type.base_type = BaseType::Real;
                            } else {
                                // Incompatible types (e.g. Text vs Int) -> Unknown
                                input_type.base_type = BaseType::Unknowns;
                            }
                        }
                    }
                }
            } else {
                // Handle COUNT() or invalid args
                // For COUNT(), we assume input isn't nullable, effectively.
                all_args_nullable = false;
            }

            match name.as_str() {
                // ---- core sqlite section --------
                // https://sqlite.org/lang_corefunc.html TODO: not all of it is implemented
                "COUNT" => Ok(Type {
                    base_type: BaseType::Integer,
                    nullable: false, // Correct: Count never returns NULL
                    contains_placeholder: false,
                }),

                "AVG" => Ok(Type {
                    base_type: BaseType::Real,
                    nullable: true, // Correct: AVG([]) is NULL
                    contains_placeholder: false,
                }),

                "SUM" | "MIN" | "MAX" => Ok(Type {
                    base_type: input_type.base_type,
                    nullable: true, // Correct: Aggregates on empty sets are NULL
                    contains_placeholder: false,
                }),

                // SQLite "TOTAL" is like SUM but returns 0.0 on empty set
                "TOTAL" => Ok(Type {
                    base_type: BaseType::Real,
                    nullable: false,
                    contains_placeholder: false,
                }),

                "RANDOM" => Ok(Type {
                    base_type: BaseType::Integer,
                    nullable: false,
                    contains_placeholder: false,
                }),

                "ABS" => Ok(Type {
                    base_type: input_type.base_type,
                    nullable: any_arg_nullable,
                    contains_placeholder: false,
                }),
                // Standard NULL propagation
                "LENGTH" | "OCTET_LENGTH" | "INSTR" | "UNICODE" | "SIGN" | "GLOB" | "LIKE"
                | "ROUND" => Ok(Type {
                    base_type: if name == "ROUND" {
                        BaseType::Real
                    } else {
                        BaseType::Integer
                    },
                    nullable: any_arg_nullable,
                    contains_placeholder: false,
                }),

                // String funcs
                "LOWER" | "UPPER" | "LTRIM" | "RTRIM" | "TRIM" | "REPLACE" | "SUBSTR"
                | "SUBSTRING" | "UNISTR" | "UNISTR_QUOTE" => Ok(Type {
                    base_type: BaseType::Text,
                    nullable: any_arg_nullable,
                    contains_placeholder: false,
                }),

                "CONCAT" | "CONCAT_WS" => Ok(Type {
                    base_type: BaseType::Text,
                    nullable: false,
                    contains_placeholder: false,
                }),

                //  COALESCE is only nullable if ALL args are nullable
                "COALESCE" | "IFNULL" => Ok(Type {
                    base_type: input_type.base_type,
                    nullable: all_args_nullable,
                    contains_placeholder: false,
                }),

                //-------MATH SECITON-------------
                // https://sqlite.org/lang_mathfunc.html
                "PI" => Ok(Type {
                    base_type: BaseType::Real,
                    nullable: false, // PI is never NULL
                    contains_placeholder: false,
                }),

                // NEVER return null as it is defined for ALL REAL numbers
                "ASINH" | "ATAN" | "ATAN2" | "COSH" | "SINH" | "TANH" | "EXP" | "DEGREES"
                | "RADIANS" => Ok(Type {
                    base_type: BaseType::Real,
                    nullable: any_arg_nullable,
                    contains_placeholder: false,
                }),

                // can return null cuz these functions are not definde for all real numbers
                "ACOS" | "ACOSH" | "ASIN" | "ATANH" | "COS" | "SIN" | "TAN" | "LN" | "LOG"
                | "LOG10" | "LOG2" | "POW" | "POWER" | "SQRT" => Ok(Type {
                    base_type: BaseType::Real,
                    nullable: true, // Always True, because Math Errors = NULL
                    contains_placeholder: false,
                }),

                // i know ceil and floor wont go through but no harm adding it
                "CEIL" | "CEILING" | "FLOOR" | "TRUNC" => Ok(Type {
                    base_type: BaseType::Real,
                    nullable: any_arg_nullable,
                    contains_placeholder: false,
                }),

                // MOD(X,Y) returns the type of X/Y.
                // If inputs are Int, result is Int. If inputs are Float, result is Float.
                "MOD" => Ok(Type {
                    base_type: input_type.base_type,
                    nullable: any_arg_nullable,
                    contains_placeholder: false,
                }),

                // --- DateTime functions ---
                // https://sqlite.org/lang_datefunc.html
                // note: returns NULL if date format is invalid for all datetime funcions
                "DATE" | "TIME" | "DATETIME" | "STRFTIME" | "TIMEDIFF" => Ok(Type {
                    base_type: BaseType::Text,
                    nullable: true,
                    contains_placeholder: false,
                }),

                "JULIANDAY" => Ok(Type {
                    base_type: BaseType::Real,
                    nullable: true,
                    contains_placeholder: false,
                }),

                "UNIXEPOCH" => Ok(Type {
                    base_type: BaseType::Integer,
                    nullable: true,
                    contains_placeholder: false,
                }),

                // -- window functions --
                // https://www.postgresql.org/docs/current/functions-window.html (note sqlite window functions are aken from postgres so need worry)

                // Integer Ranking Functions (Always non-null)
                "ROW_NUMBER" | "RANK" | "DENSE_RANK" => Ok(Type {
                    base_type: BaseType::Integer,
                    nullable: false,
                    contains_placeholder: false,
                }),

                // NTILE takes an argument. If arg is valid, it returns Int.
                "NTILE" => Ok(Type {
                    base_type: BaseType::Integer,
                    nullable: any_arg_nullable,
                    contains_placeholder: false,
                }),

                // Statistical Ranking (Always Real, between 0 and 1)
                "PERCENT_RANK" | "CUME_DIST" => Ok(Type {
                    base_type: BaseType::Real,
                    nullable: false,
                    contains_placeholder: false,
                }),

                // Value Functions (Offset)
                // LEAD/LAG return the type of the expression being tracked.
                // They return NULL if the offset is out of bounds (unless default is provided).
                // Since we can't easily check the default value type here, nullable: true is safest. TODO
                "LEAD" | "LAG" | "FIRST_VALUE" | "LAST_VALUE" | "NTH_VALUE" => Ok(Type {
                    base_type: input_type.base_type, // Inferred from the 1st argument
                    nullable: true,
                    contains_placeholder: false,
                }),

                _ => Err(format!("invalid {}", name.as_str())),
            }
        }

        // Note: these are special functions that cannot be placed in the geenric Function expression due to how sqlparser works
        // version 0.59.0

        // Math functions
        Expr::Ceil { expr, .. } | Expr::Floor { expr, .. } => {
            let input = evaluate_expr_type(expr, table_names_from_select, all_tables)?;
            Ok(Type {
                base_type: BaseType::Real, // Always float
                nullable: input.nullable,  // Null propagates
                contains_placeholder: false,
            })
        }

        // String functions (technically can be used for int and real. must give user flexibility in this case)
        Expr::Substring { expr, .. } | Expr::Trim { expr, .. } => {
            let input = evaluate_expr_type(expr, table_names_from_select, all_tables)?;
            Ok(Type {
                base_type: BaseType::Text,
                nullable: input.nullable,
                contains_placeholder: false,
            })
        }

        // TODO not too sure whether correct his part was geenrated by ai. pls come and check again.
        Expr::Case {
            conditions,
            else_result,
            ..
        } => {
            // FIX: Initialize nullable to false. We only flip it to true if necessary.
            let mut output_type = Type {
                base_type: BaseType::Null,
                nullable: false,
                contains_placeholder: false,
            };

            let mut result_types = Vec::new();
            for cond in conditions {
                result_types.push(evaluate_expr_type(
                    &cond.result,
                    table_names_from_select,
                    all_tables,
                )?);
            }

            if let Some(else_expr) = else_result {
                result_types.push(evaluate_expr_type(
                    else_expr,
                    table_names_from_select,
                    all_tables,
                )?);
            } else {
                output_type.nullable = true;
            }

            for t in result_types {
                if t.nullable {
                    output_type.nullable = true;
                }

                // Logic to merge BaseTypes
                if output_type.base_type == BaseType::Null {
                    output_type.base_type = t.base_type;
                } else if t.base_type != BaseType::Null && output_type.base_type != t.base_type {
                    match (output_type.base_type, t.base_type) {
                        (BaseType::Integer, BaseType::Real) => {
                            output_type.base_type = BaseType::Real
                        }
                        (BaseType::Real, BaseType::Integer) => {
                            output_type.base_type = BaseType::Real
                        }
                        (left, right) => {
                            return Err(format!(
                                "Incompatible types in CASE: {:?} and {:?}",
                                left, right
                            ));
                        }
                    }
                }
            }

            Ok(output_type)
        }
        _ => Err(format!("Invlaid {expr}")),
    }
}
