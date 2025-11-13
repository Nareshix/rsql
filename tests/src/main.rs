use macros::SqlMapping;
use rsql::internal_sqlite::connection::Connection;


#[derive(Debug, SqlMapping)]
struct Person {
    id: i32,
    username: String,
    email: String,
}

fn main() {
    let conn = Connection::open("hi.db").unwrap();

    let statement = conn.prepare("SELECT * FROM users").unwrap();

    for person in statement.query(Person) {
        println!("Found user: {:?}", person.id);
        println!("Found user: {:?}", person.username);
        println!("Found user: {:?}", person.email);
    }

    for person in statement.query(Person) {
        println!("{:?}", person);
    }
}
