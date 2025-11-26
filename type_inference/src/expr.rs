use std::collections::HashMap;

use sqlparser::ast::{BinaryOperator, Expr, Value};

use crate::table::TableSchema;
// TODO, need to handle cases when it can be NULL
#[derive(Debug, Clone)]
pub enum Type {
    Int,
    Float,
    Bool,
    String,
    Null,
    /// unable to infer
    Unknown,
}


/// if either type is a float, returns **Float**. Or else, it returns **Int**
fn derive_math_type(left: Type, right: Type) -> Type {
    if matches!(left, Type::Float) || matches!(right, Type::Float){
        return Type::Float
    }
    Type::Int
}


pub fn infer_type(expr: &Expr, schema: &TableSchema) -> Type {
    //TODO optimise it so it dosent create new hasmap eveyritme it recurses
    let mut tables = HashMap::new();
    tables.insert(schema.table_name.clone(), schema.columns.clone());

    match expr {

        Expr::Identifier(ident) => {
            let col_name = &ident.value;
            for col in &schema.columns {
                if col.name == *col_name {
                    return col.data_type.clone();
                }
            }
            Type::Unknown 
        },

        Expr::CompoundIdentifier(_) => todo!(),

        
        // Raw Values e.g. SELECT 1 or SELECT "hello"
        Expr::Value(val) =>{
            
            // identifies whether its a float or int
            let numeral = &val.value;
             match numeral {
                Value::Number(num, _) => {
                    // TODO scientific notation
                    if num.contains("."){
                        return Type::Float
                    }
                    Type::Int
                } 
                Value::SingleQuotedString(_) => Type::String,
                Value::DoubleQuotedString(_) => Type::String,
                Value::Boolean(_) => Type::Bool,
                Value::Null => Type::Null,
                Value::Placeholder(_) => todo!(),

                _ => Type::Unknown,
            
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
        | Expr::IsNotDistinctFrom(.. )
        | Expr::IsNormalized {..} // i dont think sqlite has TODO
        | Expr::IsUnknown(..) // i dont think sqlite has TODO
        | Expr::IsNotUnknown(..) // i dont think sqlite has TODO
        | Expr::InSubquery { .. }
        | Expr::InUnnest { .. } // i dont think sqlite has TODO
        | Expr::Exists { .. }
        // ---
        |Expr::Like { .. }
        | Expr::SimilarTo { .. } // TODO does qlite support
        | Expr::Between { .. }
        | Expr::InList { .. }
        | Expr::AnyOp { .. }
        | Expr::AllOp {.. } => Type::Bool,

        Expr::BinaryOp { left, op, right } => {
            let left_type = infer_type(left, schema);
            let right_type = infer_type(right, schema);

            match op {
                // Comparisons always return Bool
                BinaryOperator::Eq
                | BinaryOperator::NotEq
                | BinaryOperator::Gt
                | BinaryOperator::Lt
                | BinaryOperator::GtEq
                | BinaryOperator::LtEq
                | BinaryOperator::And
                | BinaryOperator::Or => Type::Bool,

                BinaryOperator::Plus
                | BinaryOperator::Minus
                | BinaryOperator::Multiply
                | BinaryOperator::Modulo
                | BinaryOperator::Divide => derive_math_type(left_type, right_type),

                // String concat always returns string
                BinaryOperator::StringConcat => Type::String,

                // TODO bitwise operation in BinaryOperator
                // TODO REGEXP. it is sqlite specific                

                _ => Type::Unknown,
            }
        }

        Expr::UnaryOp { op, expr } => {
            // op is one the 3, (+, -, NOT)
            match op {

            // +, -
            sqlparser::ast::UnaryOperator::Plus 
            | sqlparser::ast::UnaryOperator::Minus => infer_type(expr, schema),

            // <NOT> always returns Bool
            sqlparser::ast::UnaryOperator::Not => Type::Bool,

                _ => Type::Unknown
            }
        }

        // Nested expression e.g. (foo > bar) or (1)
        Expr::Nested(inner_expr) => infer_type(inner_expr, schema),

        Expr::Cast { data_type, .. } => {
            match data_type {
                sqlparser::ast::DataType::SmallInt(_) |
                sqlparser::ast::DataType::Int(_) |
                sqlparser::ast::DataType::Integer(_) |
                sqlparser::ast::DataType::BigInt(_) |
                sqlparser::ast::DataType::TinyInt(_) => Type::Int,

                sqlparser::ast::DataType::Float(_) |
                sqlparser::ast::DataType::Real |
                sqlparser::ast::DataType::Double(_) | 
                sqlparser::ast::DataType::DoublePrecision => Type::Float,

                sqlparser::ast::DataType::Boolean => Type::Bool,

                sqlparser::ast::DataType::Text |
                sqlparser::ast::DataType::String(_) |
                sqlparser::ast::DataType::Varchar(_) | 
                sqlparser::ast::DataType::Char(_) => Type::String,

                _ => Type::Unknown,
            }
        }
        //TODO functions
        // Expr::Function(func) => {
        //     let name = func.name.to_string().to_uppercase();
        //     match name.as_str() {
        //         "COUNT" => Type::Int,
        //         "SUM" | "AVG" | "MIN" | "MAX" => {
        //             // Usually depends on the argument type
        //             let arg = &func.args[0]; // Simplify: get first arg
        //             // You would recursively resolve the arg here
        //             Type::Int
        //         }
        //         _ => Type::Unknown,
        //     }
        // }

        _ => Type::Unknown,
    }
}
