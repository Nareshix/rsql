use std::collections::{HashMap, HashSet};

use sqlparser::{ast::Statement, dialect::SQLiteDialect, parser::Parser};

use crate::table::{ColumnInfo, normalize_identifier};

pub mod binding_patterns;
pub mod expr;
pub mod pg_type_cast_to_sqlite;
pub mod select_patterns;
pub mod table;

pub fn validate_insert_strict(
    sql: &str,
    tables: &HashMap<String, Vec<ColumnInfo>>,
) -> Result<(), String> {
    let dialect = SQLiteDialect {};
    let ast = Parser::parse_sql(&dialect, sql).map_err(|e| e.to_string())?;

    for statement in ast {
        if let Statement::Insert(insert) = statement {
            let raw_table_name = insert.table.to_string();

            // Normalize table name (handle "public.users" -> "users")
            let t_name = raw_table_name
                .split('.')
                .next_back()
                .unwrap_or(&raw_table_name)
                .to_lowercase();

            let schema_cols = match tables.get(&t_name) {
                Some(cols) => cols,
                None => return Err(format!("Table '{}' does not exist", t_name)),
            };

            // Implicit Insert (No columns specified) are allowed
            if insert.columns.is_empty() {
                continue;
            }

            let provided_names = insert.columns.iter().map(normalize_identifier).collect();

            let mandatory_names: HashSet<_> = schema_cols
                .iter()
                .filter(|col| !col.has_default)
                .map(|col| col.name.clone())
                .collect();

            let missing: Vec<_> = mandatory_names.difference(&provided_names).collect();

            if !missing.is_empty() {
                return Err(format!(
                    "Missing mandatory columns (columns with no default/autoincrement): {:?}",
                    missing
                ));
            }
        }
    }

    Ok(())
}
