use std::collections::HashMap;

use sqlparser::ast::{
    BinaryOperator, DataType, Expr, FunctionArg, FunctionArgExpr, FunctionArguments, Value,
};

use crate::table::ColumnInfo;
// TODO, need to handle cases when it can be NULL
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BaseType {
    Integer,
    Real,
    Bool,
    Text,
    Null,
    Unknowns,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Type {
    pub base_type: BaseType,
    /// true if theres a chance of it being null, else false
    pub nullable: bool,
}

/// https://docs.rs/sqlparser/latest/sqlparser/ast/enum.Expr.html, version 0.59.0
///
/// we will patern match all 63 (yep...). Some of them are not supported by sqlite so they will be skipped and commented.
pub fn evaluate_expr_type(
    expr: &Expr,
    table_names_from_select: Vec<String>,
    all_tables: &HashMap<String, Vec<ColumnInfo>>,
) -> Result<Type, String> {
    match expr {

        // ident can be either col name or table name, but for our use case only focus on col name
        // We are assuming that there will strictly be one table only. If one is using more than 1 table.
        // he should be more explicit (compile time check) and would be handled in CompoundIdentifier.
        Expr::Identifier(ident) => {
            // TODO, span can possibly be used for better error handlin due to showing where the error is. keep that in mind
            let table_name = &table_names_from_select[0];
            let col_name = &ident.value;
            let column_infos = &all_tables[table_name];

            for column_info in column_infos {
                if column_info.name == *col_name {
                    return Ok(column_info.data_type.clone())
                }
            }
            Err(format!("Column '{}' not found in table '{}'", col_name, table_name))

        },

        Expr::CompoundIdentifier(idents) => {
            // We expect 2 parts, e.g., "table.column"
            let table_name = &idents[0].value;
            let col_name = &idents[1].value;

            let column_infos = &all_tables[table_name];
            for column_info in column_infos {
                if column_info.name == *col_name {
                    return Ok(column_info.data_type.clone())
                }
            }
            Err(format!("Column '{}' not found in table '{}'", col_name, table_name))

        },

        // Expr::CompoundFieldAccess {..}
        // Expr::JsonAccess {..}
        // Expr::Prefixed {.. } -- mysql specifc
        // Expr::TypedString(_) -- TODO
        // Expr::Subquery() TODO
        // Expr::GroupingSets() TODO
        // Expr::Cube() TODO
        // Expr::Rollup() TODO
        // Expr::Tuple(_) TODO
        // Expr::Struct { _ }
        // Expr::Named {..} bigquery specifc
        // Expr::Dictionary() duckdb specific
        // Expr::Map() duckdb specific
        // Expr::Array() sqlite dont have array type
        // Expr::MatchAgainst { } mysql specific
        // Expr::Wildcard() --handled in select_pattern.rs
        // Expr::QualifiedWildcard(, ) --handled in select_pattern.rs
        // Expr::OuterJoin() --handled in creation of table
        // Expr::Prior() TODO
        // Expr::Lambda() bleh
        // Expr::MemberOf() json specifc TODO

        // Compound
        // Raw Values e.g. SELECT 1 or SELECT "hello"
        Expr::Value(val) =>{

            // identifies whether its a float or int
            let value = &val.value;
             match value {
                Value::Number(num, _) => {
                    if num.contains("."){
                        return Ok(Type { base_type: BaseType::Real, nullable: false })
                    }
                    Ok(Type { base_type: BaseType::Integer, nullable: false })
                }
                Value::SingleQuotedString(_) => Ok(Type { base_type: BaseType::Text, nullable: false }),
                Value::DoubleQuotedString(_) => Ok(Type { base_type: BaseType::Text, nullable: false }),
                Value::Boolean(_) => Ok(Type { base_type: BaseType::Bool, nullable: false }),
                Value::Null => Ok(Type { base_type: BaseType::Null, nullable: true }),
                Value::Placeholder(_) => todo!(),

                _ => Err(format!("{value} is an invalid type. Make sure it is TEXT, INTEGER or REAL"))

        }
        },

        // these always return a bool, regardless of input.
        Expr::IsNull(_)
        | Expr::IsNotNull(_)
        | Expr::IsTrue(_)
        | Expr::IsFalse(_)
        | Expr::IsNotFalse(_)
        | Expr::IsNotTrue(_)
        | Expr::IsDistinctFrom(..)
        | Expr::IsNotDistinctFrom(.. )
        | Expr::IsNormalized {..} // i dont think sqlite has TODO
        | Expr::IsUnknown(..) // i dont think sqlite has TODO
        | Expr::IsNotUnknown(..) // i dont think sqlite has TODO
        | Expr::Exists { .. } => Ok(Type { base_type: BaseType::Bool, nullable: false }),

        // Expr::ILike { .. }
        // Expr::RLike { .. }
        Expr::Like { .. }
        | Expr::SimilarTo { .. } // TODO does sqlite support
        | Expr::Between { .. }
        | Expr::InList { .. }
        | Expr::AnyOp { .. }
        | Expr::InSubquery { .. }
        | Expr::InUnnest { .. } // i dont think sqlite has TODO
        | Expr::AllOp {.. } => Ok(Type { base_type: BaseType::Bool, nullable: true }),

        Expr::BinaryOp { left, op, right } => {
            let left_type = evaluate_expr_type(left, table_names_from_select.clone(), all_tables)?;
            let right_type = evaluate_expr_type(right, table_names_from_select, all_tables)?;

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
                    let is_compatible = match (&left_type.base_type, &right_type.base_type) {
                    (BaseType::Integer, BaseType::Integer) => true,
                    (BaseType::Real, BaseType::Real) => true,
                    (BaseType::Text, BaseType::Text) => true,
                    (BaseType::Bool, BaseType::Bool) => true,
                    // Allow comparing Int vs Real
                    (BaseType::Integer, BaseType::Real) | (BaseType::Real, BaseType::Integer) => true,
                    (BaseType::Null, _) | (_, BaseType::Null) => true, // Null compares to anything usually
                    _ => false,
                };

                if !is_compatible {
                        Err(format!(
                            "Cannot compare Types  '{:?}' and '{:?}'.",
                            left_type.base_type, right_type.base_type                        ))

                } else {
                    Ok(Type { base_type: BaseType::Bool, nullable: true })
                }

                },

                BinaryOperator::Plus
                | BinaryOperator::Minus
                | BinaryOperator::Multiply
                | BinaryOperator::Modulo
                | BinaryOperator::Divide => {

                    // division and mod can result in NULL (when divided/mod by 0)
                    let nullable = if *op == BinaryOperator::Divide || *op == BinaryOperator::Modulo {
                        true
                    } else {
                        left_type.nullable || right_type.nullable
                    };


                    let base = match (&left_type.base_type, &right_type.base_type) {
                        (BaseType::Null, _) | (_, BaseType::Null) => BaseType::Null,
                        // Only allow numeric combinations
                        (BaseType::Real, BaseType::Real)
                        | (BaseType::Real, BaseType::Integer)
                        | (BaseType::Integer, BaseType::Real) => BaseType::Real,
                        (BaseType::Integer, BaseType::Integer) => BaseType::Integer,
                        _ => BaseType::Unknowns
                    };

                    if base == BaseType::Unknowns {
                    return Err(format!("Cannot apply operator '{:?}' to types '{:?}' and '{:?}'. Arithmetic requires numeric types.",
                            op, left_type.base_type, right_type.base_type
                            //TODO left, right
                        ))

                    }
                    Ok(Type {
                        base_type: base,
                        nullable,
                    })
                },

                // String concat always returns string
                BinaryOperator::StringConcat => Ok(Type { base_type: BaseType::Text, nullable: true }),

                // TODO bitwise operation in BinaryOperator
                // TODO REGEXP. it is sqlite specific

                _ => Err(format!("invalid {expr}"))
            }
        },

        Expr::UnaryOp { op, expr } => {
            // op is one the 3, (+, -, NOT)
            match op {

            // +, -
            sqlparser::ast::UnaryOperator::Plus
            | sqlparser::ast::UnaryOperator::Minus => evaluate_expr_type(expr, table_names_from_select, all_tables),

            sqlparser::ast::UnaryOperator::Not => evaluate_expr_type(expr, table_names_from_select, all_tables),

                _ => Err(format!("invalid {expr}"))
            }
        },

        // Nested expression e.g. (foo > bar) or (1)
        Expr::Nested(inner_expr) => evaluate_expr_type(inner_expr, table_names_from_select, all_tables),

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
                | DataType::Int8(_) => Ok(Type { base_type: BaseType::Integer, nullable: true }),

                DataType::Character(_)
                | DataType::Varchar(_)
                | DataType::CharVarying(_)
                | DataType::CharacterVarying(_)
                // sqlparser does not have  NCHAR(55)
                // sqlparser does not have  NATIVE CHARACTER(70)
                | DataType::Nvarchar(_)
                | DataType::Text
                | DataType::Clob(_) => Ok(Type { base_type: BaseType::Text, nullable: true }),

                // TODO
                // DataType::Blob(_) =>

                DataType::Real
                | DataType::Double(_)
                | DataType::DoublePrecision
                | DataType::Numeric(_) //undocumented but works
                | DataType::Decimal(_) //undocumented but works
                | DataType::Float(_) => Ok(Type { base_type: BaseType::Real, nullable: true }),


                // TODO Numeric

                _ => Err(format!("invalid data type {}", data_type))

            }
        },


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

            let mut input_type = Type { base_type: BaseType::Unknowns, nullable: false };
            let mut any_arg_nullable = false;
            let mut all_args_nullable = true; // track for COALESCE and ifnull

            if let FunctionArguments::List(list) = &func.args {
                for arg in &list.args {
                    if let FunctionArg::Unnamed(FunctionArgExpr::Expr(expr)) = arg {
                        let arg_type = evaluate_expr_type(expr, table_names_from_select.clone(), all_tables)?;

                        if arg_type.nullable {
                            any_arg_nullable = true;
                        } else {
                            all_args_nullable = false;
                        }

                        // If we already have a type, and the new arg is different (and neither are null), it's Unknown.
                        if input_type.base_type == BaseType::Null || input_type.base_type == BaseType::Unknowns {
                             // Initialize type from first non-null arg
                            input_type = arg_type;
                        } else if arg_type.base_type != BaseType::Null && input_type.base_type != arg_type.base_type {
                            // Allow Int -> Real promotion
                            if (input_type.base_type == BaseType::Integer && arg_type.base_type == BaseType::Real) ||
                               (input_type.base_type == BaseType::Real && arg_type.base_type == BaseType::Integer) {
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
                    nullable: false // Correct: Count never returns NULL
                }),

                "AVG" => Ok(Type {
                    base_type: BaseType::Real,
                    nullable: true // Correct: AVG([]) is NULL
                }),

                "SUM" | "MIN" | "MAX" => Ok(Type {
                    base_type: input_type.base_type,
                    nullable: true // Correct: Aggregates on empty sets are NULL
                }),

                // SQLite "TOTAL" is like SUM but returns 0.0 on empty set
                "TOTAL" => Ok(Type {
                    base_type: BaseType::Real,
                    nullable: false
                }),

                "RANDOM" => Ok(Type { base_type: BaseType::Integer, nullable: false }),

                // Standard NULL propagation
                "LENGTH" | "OCTET_LENGTH" | "INSTR" | "UNICODE" |
                "SIGN" | "GLOB" | "LIKE" | "ABS" | "ROUND" => Ok(Type {
                    base_type: if name == "ROUND" { BaseType::Real } else { BaseType::Integer },
                    nullable: any_arg_nullable
                }),

                // String funcs
                "LOWER" | "UPPER" | "LTRIM" | "RTRIM" | "TRIM" |
                "REPLACE" | "SUBSTR" | "SUBSTRING" | "UNISTR" | "UNISTR_QUOTE" => Ok(Type {
                    base_type: BaseType::Text,
                    nullable: any_arg_nullable
                }),

                "CONCAT" | "CONCAT_WS" => Ok(Type {
                    base_type: BaseType::Text,
                    nullable: false
                }),

                //  COALESCE is only nullable if ALL args are nullable
                "COALESCE" | "IFNULL" => Ok(Type {
                    base_type: input_type.base_type,
                    nullable: all_args_nullable
                }),


                //-------MATH SECITON-------------
                // https://sqlite.org/lang_mathfunc.html
                "PI" => Ok(Type {
                    base_type: BaseType::Real,
                    nullable: false // PI is never NULL
                }),


                // NEVER return null as it is defined for ALL REAL numbers
                "ASINH" | "ATAN" | "ATAN2" |
                "COSH" | "SINH" | "TANH" |
                "EXP" | "DEGREES" | "RADIANS" => Ok(Type {
                    base_type: BaseType::Real,
                    nullable: any_arg_nullable
                }),

                // can return null cuz these functions are not definde for all real numbers
                "ACOS" | "ACOSH" | "ASIN" | "ATANH" |
                "COS" | "SIN" | "TAN" |
                "LN" | "LOG" | "LOG10" | "LOG2" |
                "POW" | "POWER" | "SQRT" => Ok(Type {
                    base_type: BaseType::Real,
                    nullable: true // Always True, because Math Errors = NULL
                }),


                // i know ceil and floor wont go through but no harm adding it
                "CEIL" | "CEILING" | "FLOOR" | "TRUNC" => Ok(Type {
                    base_type: BaseType::Real,
                    nullable: any_arg_nullable
                }),

                // MOD(X,Y) returns the type of X/Y.
                // If inputs are Int, result is Int. If inputs are Float, result is Float.
                "MOD" => Ok(Type {
                    base_type: input_type.base_type,
                    nullable: any_arg_nullable
                }),


                // --- DateTime functions ---
                // https://sqlite.org/lang_datefunc.html
                // note: returns NULL if date format is invalid for all datetime funcions

                "DATE" | "TIME" | "DATETIME" | "STRFTIME" | "TIMEDIFF" => Ok(Type {
                    base_type: BaseType::Text,
                    nullable: true
                }),

                "JULIANDAY" => Ok(Type {
                    base_type: BaseType::Real,
                    nullable: true
                }),

                "UNIXEPOCH" => Ok(Type {
                    base_type: BaseType::Integer,
                    nullable: true
                }),


            // -- window functions --
            // https://www.postgresql.org/docs/current/functions-window.html (note sqlite window functions are aken from postgres so need worry)

            // Integer Ranking Functions (Always non-null)
            "ROW_NUMBER" | "RANK" | "DENSE_RANK" => Ok(Type {
                base_type: BaseType::Integer,
                nullable: false
            }),

            // NTILE takes an argument. If arg is valid, it returns Int.
            "NTILE" => Ok(Type {
                base_type: BaseType::Integer,
                nullable: any_arg_nullable
            }),

            // Statistical Ranking (Always Real, between 0 and 1)
            "PERCENT_RANK" | "CUME_DIST" => Ok(Type {
                base_type: BaseType::Real,
                nullable: false
            }),

            // Value Functions (Offset)
            // LEAD/LAG return the type of the expression being tracked.
            // They return NULL if the offset is out of bounds (unless default is provided).
            // Since we can't easily check the default value type here, nullable: true is safest. TODO
            "LEAD" | "LAG" | "FIRST_VALUE" | "LAST_VALUE" | "NTH_VALUE" => Ok(Type {
                base_type: input_type.base_type, // Inferred from the 1st argument
                nullable: true
            }),

                _ => Err(format!("invalid {}", name.as_str()))
            }
        }



        // Note: these are special functions that cannot be placed in the geenric Function expression due to how sqlparser works
        // version 0.59.0

        // Math functions
        Expr::Ceil { expr, .. } | Expr::Floor { expr, .. } => {
            let input = evaluate_expr_type(expr, table_names_from_select, all_tables)?;
            Ok(Type {
                base_type: BaseType::Real, // Always float
                nullable: input.nullable   // Null propagates
            })
        }

        // String functions (technically can be used for int and real. must give user flexibility in this case)
        Expr::Substring { expr, .. } | Expr::Trim { expr, .. } => {
            let input = evaluate_expr_type(expr, table_names_from_select, all_tables)?;
            Ok(Type {
                base_type: BaseType::Text,
                nullable: input.nullable
            })
        }

        // TODO not too sure whether correct his part was geenrated by ai. pls come and check again.

    Expr::Case { conditions, else_result, .. } => {
            let mut output_type = Type { base_type: BaseType::Null, nullable: true };

            // 1. Collect all outcome types (THEN ... and ELSE ...)
            let mut result_types = Vec::new();
            for cond in conditions {
                result_types.push(evaluate_expr_type(&cond.result, table_names_from_select.clone(), all_tables)?);
            }

            if let Some(else_expr) = else_result {
                result_types.push(evaluate_expr_type(else_expr, table_names_from_select.clone(), all_tables)?);
            } else {
                // No ELSE means implicit "ELSE NULL", making the result nullable
                output_type.nullable = true;
            }

            // 2. Unify types
            for t in result_types {
                // If any branch is nullable, the whole result is nullable
                if t.nullable {
                    output_type.nullable = true;
                }

                // Logic to merge BaseTypes
                if output_type.base_type == BaseType::Null {
                    // Initialize with first non-null type found
                    output_type.base_type = t.base_type;
                } else if t.base_type != BaseType::Null && output_type.base_type != t.base_type {
                    // Type Promotion: Int + Real = Real
                    match (output_type.base_type, t.base_type) {
                        (BaseType::Integer, BaseType::Real) => output_type.base_type = BaseType::Real,
                        (BaseType::Real, BaseType::Integer) => output_type.base_type = BaseType::Real,
                        // If types are totally different (Text vs Int), it's an error
                        (left, right) => return Err(format!("Incompatible types in CASE: {:?} and {:?}", left, right)),
                    }
                }
            }

            Ok(output_type)
        },
        _ => Err(format!("Invlaid {expr}"))
    }
}
