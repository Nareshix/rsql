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

use rsql::internal_sqlite::connection::Connection;

struct Person {
    id: i32,
    username: String,
    email: String,
}

fn main() {
    let conn = Connection::open("hi.db").unwrap();
    //     let y = x
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

    for row_result in statement.query() {
        println!("Found user: {:?}", row_result.id);
    };
}
