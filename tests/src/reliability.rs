use std::time::Instant;

use rsql::SqlMapping;
use rsql::internal_sqlite::ergonomic::connection::Connection;

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
    let conn = Connection::open_memory().unwrap();

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
        "CREATE TABLE addresses (
    address_id    INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id       INTEGER NOT NULL,
    address_line  TEXT NOT NULL,
    city          TEXT NOT NULL,
    country       TEXT NOT NULL,
    is_primary    BOOLEAN DEFAULT 0,
    FOREIGN KEY (user_id) REFERENCES users(user_id)
);

",
    )
    .unwrap()
    .step();

    conn.prepare(
        "-- CATEGORIES TABLE (self-referencing for hierarchy)
CREATE TABLE categories (
    category_id   INTEGER PRIMARY KEY AUTOINCREMENT,
    name          TEXT NOT NULL,
    parent_id     INTEGER,
    FOREIGN KEY (parent_id) REFERENCES categories(category_id)
);
",
    )
    .unwrap()
    .step();

    conn.prepare(
        "CREATE TABLE products (
    product_id    INTEGER PRIMARY KEY AUTOINCREMENT,
    category_id   INTEGER NOT NULL,
    name          TEXT NOT NULL,
    price         REAL NOT NULL CHECK(price > 0),
    stock         INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (category_id) REFERENCES categories(category_id)
);
",
    )
    .unwrap()
    .step();

    conn.prepare(
        "-- ORDERS
CREATE TABLE orders (
    order_id      INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id       INTEGER NOT NULL,
    order_time    TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    status        TEXT NOT NULL CHECK(status IN ('pending','paid','shipped','cancelled')),
    FOREIGN KEY (user_id) REFERENCES users(user_id)
);",
    )
    .unwrap()
    .step();

    conn.prepare(
        "-- ORDER ITEMS (many-to-many between orders & products)
CREATE TABLE order_items (
    item_id       INTEGER PRIMARY KEY AUTOINCREMENT,
    order_id      INTEGER NOT NULL,
    product_id    INTEGER NOT NULL,
    quantity      INTEGER NOT NULL CHECK(quantity > 0),
    price_each    REAL NOT NULL CHECK(price_each > 0),
    FOREIGN KEY (order_id) REFERENCES orders(order_id),
    FOREIGN KEY (product_id) REFERENCES products(product_id)
);",
    )
    .unwrap()
    .step();

    conn.prepare(
        "INSERT INTO users (username, email) VALUES
('alice', 'alice@example.com'),
('bob',   'bob@example.com'),
('charlie','charlie@example.com');",
    )
    .unwrap()
    .step();

    conn.prepare(
        "INSERT INTO addresses (user_id, address_line, city, country, is_primary) VALUES
(1, '123 Street A', 'Singapore', 'SG', 1),
(1, '456 Backup Ave', 'Tokyo', 'JP', 0),
(2, '789 Main Road', 'London', 'UK', 1);",
    )
    .unwrap()
    .step();

    conn.prepare(
        "INSERT INTO categories (name, parent_id) VALUES
('Electronics', NULL),
('Computers', 1),
('Phones', 1),
('Accessories', 1);",
    )
    .unwrap()
    .step();

    conn.prepare(
        "INSERT INTO products (category_id, name, price, stock) VALUES
(2, 'Laptop Pro 15',   2500, 10),
(2, 'Gaming PC X',     3200, 5),
(3, 'Smartphone Z',    999,  25),
(4, 'USB-C Cable',     15,   200);",
    )
    .unwrap()
    .step();

    conn.prepare(
        "INSERT INTO orders (user_id, status) VALUES
(1, 'pending'),
(1, 'paid'),
(2, 'shipped');",
    )
    .unwrap()
    .step();

    conn.prepare(
        "INSERT INTO order_items (order_id, product_id, quantity, price_each) VALUES
(1, 1, 1, 2500),
(1, 4, 3, 15),
(2, 3, 1, 999),
(3, 4, 2, 15);",
    )
    .unwrap()
    .step();

    conn.prepare(
        "UPDATE products
SET price = price * 1.10
WHERE product_id = 1;",
    )
    .unwrap()
    .step();

    conn.prepare(
        "UPDATE orders
SET status = 'shipped'
WHERE order_id = 1;",
    )
    .unwrap()
    .step();

    conn.prepare(
        "UPDATE products
SET stock = stock + 50
WHERE product_id = 4;",
    )
    .unwrap()
    .step();

    conn.prepare(
        "DELETE FROM addresses
WHERE address_id = 2;",
    )
    .unwrap()
    .step();

    conn.prepare(
        "DELETE FROM orders
WHERE status = 'cancelled';",
    )
    .unwrap()
    .step();

    #[derive(Debug, SqlMapping)]
    struct Test1 {
        order_id: i32,
        username: String,
        sum: f64,
        status: String,
    }

    let x = conn
        .prepare(
            "SELECT 
    o.order_id,
    u.username,
    SUM(oi.quantity * oi.price_each) AS total_amount,
    o.status
FROM orders o
JOIN users u ON o.user_id = u.user_id
JOIN order_items oi ON oi.order_id = o.order_id
GROUP BY o.order_id;
",
        )
        .unwrap();

    for i in x.query(Test1) {
        println!("{:?}", i?);
    }

    #[derive(Debug, SqlMapping)]
    struct LowStock {
        name: String,
        stock: i32,
    }

    let q1 = conn
        .prepare(
            "SELECT name, stock 
FROM products 
WHERE stock < 20;",
        )
        .unwrap();

    for i in q1.query(LowStock) {
        println!("{:?}", i?);
    }

    #[derive(Debug, SqlMapping)]
    struct CategoryHierarchy {
        category: String,
        parent_category: Option<String>,
    }

    let q2 = conn
        .prepare(
            "SELECT 
    c1.name AS category,
    c2.name AS parent_category
FROM categories c1
LEFT JOIN categories c2 ON c1.parent_id = c2.category_id;",
        )
        .unwrap();

    for i in q2.query(CategoryHierarchy) {
        println!("{:?}", i?);
    }

    #[derive(Debug, SqlMapping)]
    struct UserPrimaryAddress {
        username: String,
        address_line: String,
        city: String,
        country: String,
    }

    let q3 = conn
        .prepare(
            "SELECT 
    u.username,
    a.address_line,
    a.city,
    a.country
FROM users u
JOIN addresses a ON u.user_id = a.user_id
WHERE a.is_primary = 1;",
        )
        .unwrap();

    for i in q3.query(UserPrimaryAddress) {
        println!("{:?}", i?);
    }

    #[derive(Debug, SqlMapping)]
    struct OrdersItemCount {
        order_id: i32,
        num_items: i32,
    }

    let q4 = conn
        .prepare(
            "SELECT order_id, COUNT(*) AS num_items
                FROM order_items
                GROUP BY order_id
                HAVING COUNT(*) > ?;",
        )
        .unwrap();

    q4.bind_parameter(1, 1).unwrap();

    for i in q4.query(OrdersItemCount) {
        println!("{:?}", i?);
    }


    #[derive(Debug, SqlMapping)]
struct ProductFilter {
    product_id: i32,
    name: String,
    price: f64,
    stock: i32,
    category_id: i32,
}

let q = conn
    .prepare(
        "SELECT product_id, name, price, stock, category_id
FROM products
WHERE (stock < $1 AND price > $2)
   OR category_id = (SELECT category_id FROM categories WHERE name = $3);",
    )
    .unwrap();


    q.bind_parameter(1, 20).unwrap();
    q.bind_parameter(2, 100).unwrap();
    q.bind_parameter(3, "Computers").unwrap();
for i in q.query(ProductFilter) {
    println!("{:?}", i?);
}


    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);

    Ok(())
}


