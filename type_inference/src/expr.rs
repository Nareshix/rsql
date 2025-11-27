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

fn evaluate_expr_type(
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


        // Aggregate functions
        Expr::Function(func) => {
            let name = func.name.to_string().to_uppercase();
            match name.as_str() {
                "COUNT" => Type {
                    base_type: BaseType::Integer,
                    nullable: false, // COUNT always returns a number (0 if empty), never NULL
                },
                "SUM" | "AVG" | "MIN" | "MAX" => {
                    let arg = &func.args;

                    // closure
                    let get_arg_type = || -> Type {
                        if let FunctionArguments::List(list) = arg
                            && let Some(func_arg) = list.args.first()
                                && let FunctionArg::Unnamed(FunctionArgExpr::Expr(expr)) = func_arg {
                                    return evaluate_expr_type(expr, table_names_from_select, all_tables);
                                }
                        Type { base_type: BaseType::Null, nullable: true }
                    };

                    let input_type = get_arg_type();

                    match name.as_str() {
                        // AVG always produces Real
                        "AVG" => Type {
                            base_type: BaseType::Real,
                            nullable: true,
                        },

                        "SUM" => Type {
                            base_type: input_type.base_type,
                            nullable: true,
                        },

                        "MIN" | "MAX" => Type {
                            base_type: input_type.base_type,
                            nullable: true,
                        },
                        _ => unreachable!(),
                    }
                }
                _ => Type {
                    base_type: BaseType::Null,
                    nullable: false,
                },
            }
        },

        _ => Type { base_type: BaseType::Null, nullable: false },
    }
}
