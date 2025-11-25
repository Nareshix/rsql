mod table;
use crate::table::create_table;

fn main() {
    let sql_create_table = "
        CREATE TABLE users (
            id INTEGER PRIMARY KEY, 
            name TEXT AUTOINCREMENT, 
            age INTEGER CHECK (age >= 18),
            status TEXT CHECK (status IN ('active', 'inactive'))
        );
    ";

    
    let schema = create_table(sql_create_table).unwrap();
    println!("{:#?}", schema);
}
