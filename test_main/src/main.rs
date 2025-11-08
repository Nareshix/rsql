// []
// struct Person {
//     id: i32,
//     name: String,
//     data: Option<Vec<u8>>,
// }

// data = conn.execute(Person, "select * from Person", [])
// for i in data{
//     print(i.id)
//     print(i.name)
//     print(i.data)
// }

// data = conn.execute(Person, "Select * from person where name=?", [name])

// for i in data{
//     print(i.id)
//     print(i.name)
//     print(i.data)
// }

use std::ffi::CStr;

use libsqlite3_sys::{sqlite3_column_int, sqlite3_column_text, sqlite3_stmt};
use rsql::{
    internal_sqlite::connection::Connection,
    traits::from_sql::{FromSql, RowMapper},
};

#[derive(Debug)]
pub struct Person {
    id: i32,
    username: String,
    email: String,
}
struct PersonMapper;

impl RowMapper for PersonMapper {
    // We explicitly associate this mapper with the Person struct.
    type Output = Person;

    // The mapping logic is moved here from the old `FromSql` impl.
    unsafe fn map_row(&self, stmt: *mut sqlite3_stmt) -> Self::Output {
        let id = unsafe { libsqlite3_sys::sqlite3_column_int(stmt, 0) };
        let c_username = unsafe { libsqlite3_sys::sqlite3_column_text(stmt, 1) } as *const i8;
        let c_email = unsafe { libsqlite3_sys::sqlite3_column_text(stmt, 2) } as *const i8;

        let username = unsafe { CStr::from_ptr(c_username).to_string_lossy().into_owned() };
        let email = unsafe { CStr::from_ptr(c_email).to_string_lossy().into_owned() };

        Person {
            id,
            username,
            email,
        }
    }
}

fn main() {
    let conn = Connection::open("his.db").unwrap();
    //     let y = conn
    //         .prepare(
    //             "CREATE TABLE users (
    //     id INTEGER PRIMARY KEY AUTOINCREMENT,
    //     username TEXT NOT NULL UNIQUE,
    //     email TEXT NOT NULL UNIQUE
    // );
    // ",
    //         )
    //         .unwrap();
    //     y.step();

    let statement = conn.prepare("SELECT * FROM users").unwrap();
    let person_mapper = PersonMapper;

    for person in statement.query(person_mapper) {
        println!("Found user: {:?}", person);
    }
}
