use std::collections::HashMap;

use crate::{expr::get_type_of_columns_from_select, table::create_table};
mod table;
mod expr;
fn main() {
    let sql = "CREATE TABLE users (id INTEGER, name TEXT, wow REAL);";
    let mut tables = HashMap::new();
    create_table(sql, &mut tables);

    // Changed SQL to explicit columns because '*' is a Wildcard, not an Expr
    let sql = "SELECT id+1, name, wow FROM users";

    get_type_of_columns_from_select(sql, &tables);

    // let ast = &Parser::parse_sql(&SQLiteDialect {}, sql).unwrap()[0];


}

