use std::time::Instant;


#[derive(Debug, rsql::SqlMapping)]
#[allow(unused)]
struct Person {
    url: String,
    caption: String,
}

// results for select * from table
// rsql Elapsed: 136.88s
// rusqlite Elapsed: 137.13s
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let now = Instant::now();

    // rsql
    let conn = rsql::Connection::open("mom.db").unwrap();

    conn.prepare(
        "CREATE TABLE users (
    user_id       INTEGER PRIMARY KEY AUTOINCREMENT,
    username      TEXT NOT NULL UNIQUE,
    email         TEXT NOT NULL UNIQUE,
    created_at    TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);",
    )
    .unwrap()
    .step();

    conn.prepare(
        "INSERT INTO users (username, email)
VALUES
    ('user1', 'user1@example.com'),
    ('user2', 'user2@example.com'),
    ('user3', 'user3@example.com'),
    ('user4', 'user4@example.com'),
    ('user5', 'user5@example.com'),
    ('user6', 'user6@example.com'),
    ('user7', 'user7@example.com'),
    ('user8', 'user8@example.com'),
    ('user9', 'user9@example.com'),
    ('user10', 'user10@example.com');
").unwrap().step();

    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);

    Ok(())
}


