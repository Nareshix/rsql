use macros::execute;
use rsql::internal_sqlite::connection::Connection;


#[derive(PartialEq, Debug)]
#[allow(unused)]
struct User;

fn main() -> Result<(), Box<dyn std::error::Error>>{
    let conn = Connection::open("hi.db")?;
    let name = "mom";
    let email = "mom@example.com";




    execute!(conn, "INSERT INTO users (username, email) VALUES (?, ?)", (name, email));

    Ok(())
}
