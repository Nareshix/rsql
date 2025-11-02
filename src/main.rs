use rsql::connection::Connection;

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

    x.prepare(
        "INSERT INTO users (username, email) VALUES
('alice', 'alice@example.com'),
('bob', 'bob@example.com'),
('charlie', 'charlie@example.com'),
('diana', 'diana@example.com'),
('eve', 'eve@example.com');
",
    )
    .unwrap()
    .step();

    
}
