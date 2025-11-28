use std::collections::HashMap;

use sqlparser::ast::{
    BinaryOperator, DataType, Expr, FunctionArg, FunctionArgExpr, FunctionArguments, Value,
};

use crate::table::ColumnInfo;
// TODO, need to handle cases when it can be NULL
#[derive(Debug, Clone, PartialEq)]
pub enum BaseType {
    Integer,
    Real,
    Bool,
    Text,
    Null,
    /// unable to infer
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Type {
    pub base_type: BaseType,
    /// true if theres a chance of it being null, else false
    pub nullable: bool,
}

#[allow(unused)]
/// if either type is a float, returns **Float**. Or else, it returns **Int**
fn derive_math_type(left: Type, right: Type) -> Type {
    let nullable = left.nullable || right.nullable;

    let base = match (&left.base_type, &right.base_type) {
        (BaseType::Null, _) | (_, BaseType::Null) => BaseType::Null,
        (BaseType::Real, _) | (_, BaseType::Real) => BaseType::Real,
        (BaseType::Integer, BaseType::Integer) => BaseType::Integer,
        _ => BaseType::Null,
    };

    Type {
        base_type: base,
        nullable,
    }
}

/// https://docs.rs/sqlparser/latest/sqlparser/ast/enum.Expr.html, version 0.59.0
///
/// we will patern match all 63 (yep...). Some of them are not supported by sqlite so they will be skipped and commented.
pub fn evaluate_expr_type(
    expr: &Expr,
    table_names_from_select: Vec<String>,
    all_tables: &HashMap<String, Vec<ColumnInfo>>,
) -> Type {
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
                    return column_info.data_type.clone()
                }
            }
            panic!("single identifier not found in tables.")
        },

        Expr::CompoundIdentifier(idents) => {
            // We expect 2 parts, e.g., "table.column"
            let table_name = &idents[0].value;
            let col_name = &idents[1].value;

            let column_infos = &all_tables[table_name];
            for column_info in column_infos {
                if column_info.name == *col_name {
                    return column_info.data_type.clone()
                }
            }
            panic!("multiple identifier not found in tables.")
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
            let numeral = &val.value;
             match numeral {
                Value::Number(num, _) => {
                    if num.contains("."){
                        return Type { base_type: BaseType::Real, nullable: false }
                    }
                    Type { base_type: BaseType::Integer, nullable: false }
                }
                Value::SingleQuotedString(_) => Type { base_type: BaseType::Text, nullable: false },
                Value::DoubleQuotedString(_) => Type { base_type: BaseType::Text, nullable: false },
                Value::Boolean(_) => Type { base_type: BaseType::Bool, nullable: false },
                Value::Null => Type { base_type: BaseType::Null, nullable: false },
                Value::Placeholder(_) => todo!(),

                _ => Type { base_type: BaseType::Null, nullable: false },

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
        | Expr::Exists { .. } => Type { base_type: BaseType::Bool, nullable: false },

        // Expr::ILike { .. }
        // Expr::RLike { .. }
        Expr::Like { .. }
        | Expr::SimilarTo { .. } // TODO does sqlite support
        | Expr::Between { .. }
        | Expr::InList { .. }
        | Expr::AnyOp { .. }
        | Expr::InSubquery { .. }
        | Expr::InUnnest { .. } // i dont think sqlite has TODO
        | Expr::AllOp {.. } => Type { base_type: BaseType::Bool, nullable: true },

        Expr::BinaryOp { left, op, right } => {
            let left_type = evaluate_expr_type(left, table_names_from_select.clone(), all_tables);
            let right_type = evaluate_expr_type(right, table_names_from_select, all_tables);

            match op {
                // Comparisons always return Bool
                BinaryOperator::Eq
                | BinaryOperator::NotEq
                | BinaryOperator::Gt
                | BinaryOperator::Lt
                | BinaryOperator::GtEq
                | BinaryOperator::LtEq
                | BinaryOperator::And
                | BinaryOperator::Or => Type { base_type: BaseType::Bool, nullable: true },

                BinaryOperator::Plus
                | BinaryOperator::Minus
                | BinaryOperator::Multiply
                | BinaryOperator::Modulo
                | BinaryOperator::Divide => derive_math_type(left_type, right_type),

                // String concat always returns string
                BinaryOperator::StringConcat => Type { base_type: BaseType::Text, nullable: true },

                // TODO bitwise operation in BinaryOperator
                // TODO REGEXP. it is sqlite specific

                _ => Type { base_type: BaseType::Null, nullable: false },
            }
        },

        Expr::UnaryOp { op, expr } => {
            // op is one the 3, (+, -, NOT)
            match op {

            // +, -
            sqlparser::ast::UnaryOperator::Plus
            | sqlparser::ast::UnaryOperator::Minus => evaluate_expr_type(expr, table_names_from_select, all_tables),

            // <NOT> always returns Bool
            sqlparser::ast::UnaryOperator::Not => Type { base_type: BaseType::Bool, nullable: true },

                _ => Type { base_type: BaseType::Null, nullable: false }
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
                | DataType::Int8(_) => Type { base_type: BaseType::Integer, nullable: true },

                DataType::Character(_)
                | DataType::Varchar(_)
                | DataType::CharVarying(_)
                | DataType::CharacterVarying(_)
                // sqlparser does not have  NCHAR(55)
                // sqlparser does not have  NATIVE CHARACTER(70)
                | DataType::Nvarchar(_)
                | DataType::Text
                | DataType::Clob(_) => Type { base_type: BaseType::Text, nullable: true },

                // TODO
                // DataType::Blob(_) =>

                DataType::Real
                | DataType::Double(_)
                | DataType::DoublePrecision
                | DataType::Float(_) => Type { base_type: BaseType::Real, nullable: true },


                // TODO Numeric

                _ => panic!("Datatype CAST Not supported by sqlite")
            }
        },


// 3. Window Functions (Ranking) TODO sqlite core, math and date also TODO

// Ranking functions generate new numbers based on row position. They cannot produce nulls.

//     ROW_NUMBER()

//     RANK()

//     DENSE_RANK()

//     NTILE()

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

            let mut input_type = Type { base_type: BaseType::Null, nullable: false };
            let mut any_arg_nullable = false;
            let mut all_args_nullable = true; // track for COALESCE and ifnull

            if let FunctionArguments::List(list) = &func.args {
                for arg in &list.args {
                    if let FunctionArg::Unnamed(FunctionArgExpr::Expr(expr)) = arg {
                        let arg_type = evaluate_expr_type(expr, table_names_from_select.clone(), all_tables);

                        if arg_type.nullable {
                            any_arg_nullable = true;
                        } else {
                            all_args_nullable = false; // Found a non-null arg
                        }

                        // Simple type coercion (Keep the first non-null/non-unknown type)
                        if input_type.base_type == BaseType::Null && arg_type.base_type != BaseType::Null {
                            input_type = arg_type;
                        }
                    }
                }
            } else {
                // Handle COUNT(*) or invalid args
                // For COUNT(*), we assume input isn't nullable, effectively.
                all_args_nullable = false;
            }

            match name.as_str() {
                // ---- core sqlite section --------
                // https://sqlite.org/lang_corefunc.html

                "COUNT" => Type {
                    base_type: BaseType::Integer,
                    nullable: false // Correct: Count never returns NULL
                },

                "AVG" => Type {
                    base_type: BaseType::Real,
                    nullable: true // Correct: AVG([]) is NULL
                },

                "SUM" | "MIN" | "MAX" => Type {
                    base_type: input_type.base_type,
                    nullable: true // Correct: Aggregates on empty sets are NULL
                },

                // SQLite "TOTAL" is like SUM but returns 0.0 on empty set
                "TOTAL" => Type {
                    base_type: BaseType::Real,
                    nullable: false
                },

                "RANDOM" => Type { base_type: BaseType::Integer, nullable: false },

                // Standard NULL propagation
                "LENGTH" | "OCTET_LENGTH" | "INSTR" | "UNICODE" |
                "SIGN" | "GLOB" | "LIKE" | "ABS" | "ROUND" => Type {
                    base_type: if name == "ROUND" { BaseType::Real } else { BaseType::Integer },
                    nullable: any_arg_nullable
                },

                // String funcs
                "LOWER" | "UPPER" | "LTRIM" | "RTRIM" | "TRIM" |
                "REPLACE" | "SUBSTR" | "SUBSTRING" | "UNISTR" | "UNISTR_QUOTE" => Type {
                    base_type: BaseType::Text,
                    nullable: any_arg_nullable
                },

                "CONCAT" | "CONCAT_WS" => Type {
                    base_type: BaseType::Text,
                    nullable: false
                },

                //  COALESCE is only nullable if ALL args are nullable
                "COALESCE" | "IFNULL" => Type {
                    base_type: input_type.base_type,
                    nullable: all_args_nullable
                },


                //-------MATH SECITON-------------
                // https://sqlite.org/lang_mathfunc.html
                "PI" => Type {
                    base_type: BaseType::Real,
                    nullable: false // PI is never NULL
                },


                // NEVER return null as it is defined for ALL REAL numbers
                "ASINH" | "ATAN" | "ATAN2" |
                "COSH" | "SINH" | "TANH" |
                "EXP" | "DEGREES" | "RADIANS" => Type {
                    base_type: BaseType::Real,
                    nullable: any_arg_nullable
                },

                // can return null cuz these functions are not definde for all real numbers
                "ACOS" | "ACOSH" | "ASIN" | "ATANH" |
                "COS" | "SIN" | "TAN" |
                "LN" | "LOG" | "LOG10" | "LOG2" |
                "POW" | "POWER" | "SQRT" => Type {
                    base_type: BaseType::Real,
                    nullable: true // Always True, because Math Errors = NULL
                },


                // i know ceil and floor wont go through but no harm adding it
                "CEIL" | "CEILING" | "FLOOR" | "TRUNC" => Type {
                    base_type: BaseType::Real,
                    nullable: any_arg_nullable
                },

                // MOD(X,Y) returns the type of X/Y.
                // If inputs are Int, result is Int. If inputs are Float, result is Float.
                "MOD" => Type {
                    base_type: input_type.base_type,
                    nullable: any_arg_nullable
                },


                // --- DateTime functions ---
                // https://sqlite.org/lang_datefunc.html
                // note: returns NULL if date format is invalid for all datetime funcions

                "DATE" | "TIME" | "DATETIME" | "STRFTIME" | "TIMEDIFF" => Type {
                    base_type: BaseType::Text,
                    nullable: true
                },

                "JULIANDAY" => Type {
                    base_type: BaseType::Real,
                    nullable: true
                },

                "UNIXEPOCH" => Type {
                    base_type: BaseType::Integer,
                    nullable: true
                },


            // -- window functions --
            // https://www.postgresql.org/docs/current/functions-window.html (note sqlite window functions are aken from postgres so need worry)

            // Integer Ranking Functions (Always non-null)
            "ROW_NUMBER" | "RANK" | "DENSE_RANK" => Type {
                base_type: BaseType::Integer,
                nullable: false
            },

            // NTILE takes an argument. If arg is valid, it returns Int.
            "NTILE" => Type {
                base_type: BaseType::Integer,
                nullable: any_arg_nullable
            },

            // Statistical Ranking (Always Real, between 0 and 1)
            "PERCENT_RANK" | "CUME_DIST" => Type {
                base_type: BaseType::Real,
                nullable: false
            },

            // Value Functions (Offset)
            // LEAD/LAG return the type of the expression being tracked.
            // They return NULL if the offset is out of bounds (unless default is provided).
            // Since we can't easily check the default value type here, nullable: true is safest. TODO
            "LEAD" | "LAG" | "FIRST_VALUE" | "LAST_VALUE" | "NTH_VALUE" => Type {
                base_type: input_type.base_type, // Inferred from the 1st argument
                nullable: true
            },

                _ => Type {
                    base_type: BaseType::Null,
                    nullable: true
                },
            }
        }



        // Note: these are special functions that cannot be placed in the geenric Function expression due to how sqlparser works
        // version 0.59.0

        // Math functions
        Expr::Ceil {expr, ..} => {
            evaluate_expr_type(expr, table_names_from_select, all_tables)
        }

        Expr::Floor { expr, .. } => {
            evaluate_expr_type(expr, table_names_from_select, all_tables)
        }

        // String functions (technically can be used for int and real. must give user flexibility in this case)
        Expr::Substring { expr,.. } => {
            evaluate_expr_type(expr, table_names_from_select, all_tables)
        }
        Expr::Trim { expr, .. } => {
            evaluate_expr_type(expr, table_names_from_select, all_tables)
        }

        _ => Type { base_type: BaseType::Null, nullable: false },
    }
}
