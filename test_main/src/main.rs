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
    name: String,
    data: Option<Vec<u8>>,
}




fn main() {
    let x = Connection::open("hi.db").unwrap();
    let y = x
        .prepare(
            "CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
",
        )
        .unwrap();
    y.step();
}