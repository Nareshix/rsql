use rsql::{SqlMapping, check, internal_sqlite::connection::Connection};

#[derive(Debug, SqlMapping)]
struct Person {
    id: i32,
    username: String,
    email: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open("hi.db").unwrap();

    // execute!(conn, "INSERT INTO ", ())

    let _ = check!("SELECT * FROM users");
    let statement = conn.prepare("SELECT * FROM users")?;

    for person in statement.query(Person) {
        println!("Found user: {:?}", person.id);
        println!("Found user: {:?}", person.username);
        println!("Found user: {:?}", person.email);
    }

    for person in statement.query(Person) {
        println!("{:?}", person);
    }

    Ok(())
}
