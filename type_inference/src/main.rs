// use crate::select_patterns::get_types_from_select;
// use crate::table::create_tables;
// use std::collections::HashMap;

mod binding_patterns;
mod expr;
mod select_patterns;
mod table;
// fn main() {
//     let mut all_tables = HashMap::new();

//     let sql = "CREATE TABLE users (id INTEGER, name TEXT, wow REAL)";
//     create_tables(sql, &mut all_tables);

//     let sql = "CREATE TABLE mom (id INTEGER, name TEXT NOT NULL, wow TEXT)";
//     create_tables(sql, &mut all_tables);

//     let sql = "SELECT * FROM mom";
//     let types = get_types_from_select(sql, &all_tables);
//     println!("{:?}", types);

//     let sql = "SELECT id > 'asd' FROM mom";
//     let types = get_types_from_select(sql, &all_tables);
//     println!("{:?}", types);

//     let sql = "SELECT not NULL";
//     let types = get_types_from_select(sql, &all_tables);
//     println!("{:?}", types);

// }

use crate::{
    binding_patterns::get_type_of_binding_parameters,
    table::create_tables,
};
use std::collections::HashMap;

fn main() {
    let mut all_tables = HashMap::new();

    let sql = "CREATE TABLE users (id INTEGER, name TEXT, wow REAL)";
    create_tables(sql, &mut all_tables);

    let sql = "CREATE TABLE mom (id INTEGER NOT NULL, name TEXT NOT NULL, wow TEXT)";
    create_tables(sql, &mut all_tables);

    // let sql = "Select 1 from users where wow in (?,?,?)";

    // let x = get_type_of_binding_parameters(sql, &all_tables);
    // println!("{:?}", x);

    // Test cases for SQL parameter binding detection

    let sql = "SELECT 1 FROM users AS u WHERE u.wow >= ?";
    let x = get_type_of_binding_parameters(sql, &mut all_tables);
    println!(" {:?}", x);

    // let sql = "SELECT * FROM users WHERE id = ?";
    // let x = get_type_of_binding_parameters(sql, &all_tables).unwrap();
    // println!("{} - {:?}", sql, x);

    // let sql = "SELECT * FROM users WHERE name = ? AND wow > ?";
    // let x = get_type_of_binding_parameters(sql, &all_tables).unwrap();
    // println!("{} - {:?}", sql, x);

    // let sql = "SELECT * FROM users WHERE wow < ? OR id != ?";
    // let x = get_type_of_binding_parameters(sql, &all_tables).unwrap();
    // println!("{} - {:?}", sql, x);

    // let sql = "UPDATE users SET wow = ? WHERE id = ?";
    // let x = get_type_of_binding_parameters(sql, &all_tables).unwrap();
    // println!("{} - {:?}", sql, x);

    // let sql = "UPDATE users SET name = ?, wow = ? WHERE id > ?";
    // let x = get_type_of_binding_parameters(sql, &all_tables).unwrap();
    // println!("{} - {:?}", sql, x);

    // let sql = "INSERT INTO users (id, name, wow) VALUES (?, ?, ?)";
    // let x = get_type_of_binding_parameters(sql, &all_tables).unwrap();
    // println!("{} - {:?}", sql, x);

    // let sql = "INSERT INTO mom (id, name, wow) VALUES (?, ?, ?)";
    // let x = get_type_of_binding_parameters(sql, &all_tables).unwrap();
    // println!("{} - {:?}", sql, x);

    // let sql = "DELETE FROM users WHERE id = ?";
    // let x = get_type_of_binding_parameters(sql, &all_tables).unwrap();
    // println!("{} - {:?}", sql, x);

    // let sql = "DELETE FROM users WHERE name = ? AND wow <= ?";
    // let x = get_type_of_binding_parameters(sql, &all_tables).unwrap();
    // println!("{} - {:?}", sql, x);

    // let sql = "SELECT * FROM users WHERE id IN (?, ?, ?)";
    // let x = get_type_of_binding_parameters(sql, &all_tables).unwrap();
    // println!("{} - {:?}", sql, x);

    // let sql = "SELECT * FROM users WHERE wow >= ? AND wow <= ?";
    // let x = get_type_of_binding_parameters(sql, &all_tables).unwrap();
    // println!("{} - {:?}", sql, x);

    // let sql = "UPDATE mom SET name = ? WHERE id = ? AND wow = ?";
    // let x = get_type_of_binding_parameters(sql, &all_tables).unwrap();
    // println!("{} - {:?}", sql, x);

    // // let sql = "SELECT * FROM mom WHERE name LIKE ?";
    // // let x = get_type_of_binding_parameters(sql, &all_tables).unwrap();
    // // println!("{} - {:?}", sql, x);

    // let sql = "SELECT * FROM users WHERE name = ? OR name = ? OR name = ?";
    // let x = get_type_of_binding_parameters(sql, &all_tables).unwrap();
    // println!("{} - {:?}", sql, x);

    // let sql = "UPDATE users SET wow = wow + ? WHERE id = ?";
    // let x = get_type_of_binding_parameters(sql, &all_tables).unwrap();
    // println!("{} - {:?}", sql, x);

    // let sql = "SELECT * FROM users WHERE id > ? AND id < ? AND wow = ?";
    // let x = get_type_of_binding_parameters(sql, &all_tables).unwrap();
    // println!("{} - {:?}", sql, x);

    // // let sql = "INSERT INTO users VALUES (?, ?, ?)";
    // // let x = get_type_of_binding_parameters(sql, &all_tables).unwrap();
    // // println!("{} - {:?}", sql, x);

    // let sql = "DELETE FROM mom WHERE id >= ? AND name != ?";
    // let x = get_type_of_binding_parameters(sql, &all_tables).unwrap();
    // println!("{} - {:?}", sql, x);

    // let sql = "SELECT name FROM users WHERE wow IS NOT NULL AND id = ?";
    // let x = get_type_of_binding_parameters(sql, &all_tables).unwrap();
    // println!("{} - {:?}", sql, x);

    // let sql = "UPDATE mom SET wow = ? WHERE id IN (?, ?, ?)";
    // let x = get_type_of_binding_parameters(sql, &all_tables).unwrap();
    // println!("{} - {:?}", sql, x);

    // let sql = "SELECT * FROM users WHERE wow = ? AND name LIKE ? AND id > ?";
    // let x = get_type_of_binding_parameters(sql, &all_tables).unwrap();
    // println!("{} - {:?}", sql, x);

    // let sql = "DELETE FROM users WHERE wow BETWEEN ? AND ? AND name = ?";
    // let x = get_type_of_binding_parameters(sql, &all_tables).unwrap();
    // println!("{} - {:?}", sql, x);

    // let sql = "SELECT COUNT(*) FROM users WHERE id < ? AND wow > ?";
    // let x = get_type_of_binding_parameters(sql, &all_tables).unwrap();
    // println!("{} - {:?}", sql, x);

    // let sql = "UPDATE users SET name = ? WHERE wow IN (?, ?) AND id != ?";
    // let x = get_type_of_binding_parameters(sql, &all_tables).unwrap();
    // println!("{} - {:?}", sql, x);

    // let sql = "SELECT * FROM mom WHERE id = ? OR (name = ? AND wow = ?)";
    // let x = get_type_of_binding_parameters(sql, &all_tables).unwrap();
    // println!("{} - {:?}", sql, x);

    // let sql = "INSERT INTO mom (id, name) VALUES (?, ?)";
    // let x = get_type_of_binding_parameters(sql, &all_tables).unwrap();
    // println!("{} - {:?}", sql, x);

    // let sql = "SELECT * FROM users WHERE wow != ? AND id NOT IN (?, ?)";
    // let x = get_type_of_binding_parameters(sql, &all_tables).unwrap();
    // println!("{} - {:?}", sql, x);

    // let sql = "UPDATE users SET wow = ?, name = ? WHERE id BETWEEN ? AND ?";
    // let x = get_type_of_binding_parameters(sql, &all_tables).unwrap();
    // println!("{} - {:?}", sql, x);

    // let sql = "DELETE FROM mom WHERE wow = ? OR id = ?";
    // let x = get_type_of_binding_parameters(sql, &all_tables).unwrap();
    // println!("{} - {:?}", sql, x);
}
