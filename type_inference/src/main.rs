use crate::select_patterns::get_types_from_select;
use crate::table::create_tables;
use std::collections::HashMap;

mod expr;
mod select_patterns;
mod table;

fn main() {
    let mut all_tables = HashMap::new();

    let sql = "CREATE TABLE users (id INTEGER, name TEXT, wow REAL)";
    create_tables(sql, &mut all_tables);

    let sql = "CREATE TABLE mom (id INTEGER, name TEXT, wow REAL)";
    create_tables(sql, &mut all_tables);

    let sql = "SELECT users.wow+ 1, mom.id / 3.0 AS asd FROM users, mom";
    let types = get_types_from_select(sql, &all_tables);
    println!("{:?}", types);

}
