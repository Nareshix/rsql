use rsql::{
    internal_sqlite::efficient::{lazy_connection::LazyConnection, lazy_statement::LazyStmt},
    lazy_sql, SqlMapping,
};
use std::time::Instant;

// -----------------------------------------------------------------------------
// 0. Dummy `sql!` macro to satisfy the compiler before the proc-macro runs
// -----------------------------------------------------------------------------
// This is a standard pattern. The `lazy_sql` proc-macro will replace
// the `sql!(...)` type, so this dummy implementation is never actually used.
#[macro_export]
macro_rules! sql {
    ($sql:literal) => {
        () // The type expands to the unit type, which is a valid placeholder.
    };
}

// -----------------------------------------------------------------------------
// 1. Result Structs (Unchanged)
// -----------------------------------------------------------------------------

#[derive(Debug, SqlMapping)]
struct Test1 {
    order_id: i32,
    username: String,
    sum: f64,
    status: String,
}

#[derive(Debug, SqlMapping)]
struct LowStock {
    name: String,
    stock: i32,
}

#[derive(Debug, SqlMapping)]
struct CategoryHierarchy {
    category: String,
    parent_category: Option<String>,
}

#[derive(Debug, SqlMapping)]
struct UserPrimaryAddress {
    username: String,
    address_line: String,
    city: String,
    country: String,
}

#[derive(Debug, SqlMapping)]
struct OrdersItemCount {
    order_id: i32,
    num_items: i32,
}

#[derive(Debug, SqlMapping)]
struct ProductFilter {
    product_id: i32,
    name: String,
    price: f64,
    stock: i32,
    category_id: i32,
}

// -----------------------------------------------------------------------------
// 2. The DAO Definition (MODIFIED to fit the original macro)
// -----------------------------------------------------------------------------
#[lazy_sql]
pub struct ShopDao { // NO <'a> lifetime here; the macro adds it.
    // NO `db` field here; the macro injects `__db: &'a LazyConnection`.

    // --- DDL ---
    create_users: sql!(
        "CREATE TABLE users (
        user_id       INTEGER PRIMARY KEY AUTOINCREMENT,
        username      TEXT NOT NULL UNIQUE,
        email         TEXT NOT NULL UNIQUE,
        created_at    TIMESTAMP DEFAULT CURRENT_TIMESTAMP
    );"
    ),

    create_addresses: sql!(
        "CREATE TABLE addresses (
        address_id    INTEGER PRIMARY KEY AUTOINCREMENT,
        user_id       INTEGER NOT NULL,
        address_line  TEXT NOT NULL,
        city          TEXT NOT NULL,
        country       TEXT NOT NULL,
        is_primary    BOOLEAN DEFAULT 0,
        FOREIGN KEY (user_id) REFERENCES users(user_id)
    );"
    ),

    create_categories: sql!(
        "CREATE TABLE categories (
        category_id   INTEGER PRIMARY KEY AUTOINCREMENT,
        name          TEXT NOT NULL,
        parent_id     INTEGER,
        FOREIGN KEY (parent_id) REFERENCES categories(category_id)
    );"
    ),

    create_products: sql!(
        "CREATE TABLE products (
        product_id    INTEGER PRIMARY KEY AUTOINCREMENT,
        category_id   INTEGER NOT NULL,
        name          TEXT NOT NULL,
        price         REAL NOT NULL CHECK(price > 0),
        stock         INTEGER NOT NULL DEFAULT 0,
        FOREIGN KEY (category_id) REFERENCES categories(category_id)
    );"
    ),

    create_orders: sql!(
        "CREATE TABLE orders (
        order_id      INTEGER PRIMARY KEY AUTOINCREMENT,
        user_id       INTEGER NOT NULL,
        order_time    TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        status        TEXT NOT NULL CHECK(status IN ('pending','paid','shipped','cancelled')),
        FOREIGN KEY (user_id) REFERENCES users(user_id)
    );"
    ),

    create_order_items: sql!(
        "CREATE TABLE order_items (
        item_id       INTEGER PRIMARY KEY AUTOINCREMENT,
        order_id      INTEGER NOT NULL,
        product_id    INTEGER NOT NULL,
        quantity      INTEGER NOT NULL CHECK(quantity > 0),
        price_each    REAL NOT NULL CHECK(price_each > 0),
        FOREIGN KEY (order_id) REFERENCES orders(order_id),
        FOREIGN KEY (product_id) REFERENCES products(product_id)
    );"
    ),

    // --- INSERTS ---
    insert_users: sql!(
        "INSERT INTO users (username, email) VALUES
        ('alice', 'alice@example.com'),
        ('bob',   'bob@example.com'),
        ('charlie','charlie@example.com');"
    ),

    insert_addresses: sql!(
        "INSERT INTO addresses (user_id, address_line, city, country, is_primary) VALUES
        (1, '123 Street A', 'Singapore', 'SG', 1),
        (1, '456 Backup Ave', 'Tokyo', 'JP', 0),
        (2, '789 Main Road', 'London', 'UK', 1);"
    ),

    insert_categories: sql!(
        "INSERT INTO categories (name, parent_id) VALUES
        ('Electronics', NULL),
        ('Computers', 1),
        ('Phones', 1),
        ('Accessories', 1);"
    ),

    insert_products: sql!(
        "INSERT INTO products (category_id, name, price, stock) VALUES
        (2, 'Laptop Pro 15',   2500, 10),
        (2, 'Gaming PC X',     3200, 5),
        (3, 'Smartphone Z',    999,  25),
        (4, 'USB-C Cable',     15,   200);"
    ),

    insert_orders: sql!(
        "INSERT INTO orders (user_id, status) VALUES
        (1, 'pending'),
        (1, 'paid'),
        (2, 'shipped');"
    ),

    insert_order_items: sql!(
        "INSERT INTO order_items (order_id, product_id, quantity, price_each) VALUES
        (1, 1, 1, 2500),
        (1, 4, 3, 15),
        (2, 3, 1, 999),
        (3, 4, 2, 15);"
    ),

    // --- UPDATES / DELETES ---
    update_price: sql!("UPDATE products SET price = price * 1.10 WHERE product_id = 1;"),
    update_order_shipped: sql!("UPDATE orders SET status = 'shipped' WHERE order_id = 1;"),
    update_stock: sql!("UPDATE products SET stock = stock + 50 WHERE product_id = 4;"),
    delete_address: sql!("DELETE FROM addresses WHERE address_id = 2;"),
    delete_cancelled: sql!("DELETE FROM orders WHERE status = 'cancelled';"),

    // --- QUERIES ---
    q_complex_join: sql!(
        "SELECT
        o.order_id,
        u.username,
        SUM(oi.quantity * oi.price_each) AS total_amount,
        o.status
    FROM orders o
    JOIN users u ON o.user_id = u.user_id
    JOIN order_items oi ON oi.order_id = o.order_id
    GROUP BY o.order_id;"
    ),

    q_low_stock: sql!("SELECT name, stock FROM products WHERE stock < 20;"),

    q_categories: sql!(
        "SELECT
        c1.name AS category,
        c2.name AS parent_category
    FROM categories c1
    LEFT JOIN categories c2 ON c1.parent_id = c2.category_id;"
    ),

    q_addresses: sql!(
        "SELECT
        u.username,
        a.address_line,
        a.city,
        a.country
    FROM users u
    JOIN addresses a ON u.user_id = a.user_id
    WHERE a.is_primary = 1;"
    ),

    q_item_count: sql!(
        "SELECT order_id, COUNT(*) AS num_items
        FROM order_items
        GROUP BY order_id
        HAVING COUNT(*) > ?;"
    ),

    q_product_filter: sql!(
        "SELECT product_id, name, price, stock, category_id
        FROM products
        WHERE (stock < ?1 AND price > ?2)
        OR category_id = (SELECT category_id FROM categories WHERE name = ?3);"
    ),
}

// -----------------------------------------------------------------------------
// 3. Main Execution (Unchanged)
// -----------------------------------------------------------------------------
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let now = Instant::now();

    // 1. Open Connection
    let conn = LazyConnection::open_memory().expect("Failed to open memory db");

    // 2. Initialize DAO
    // This still works because the macro generates `ShopDao::new(db: &'a LazyConnection)`
    let mut dao = ShopDao::new(&conn);

    println!("--- Initializing Schema & Data ---");
    // All these method calls are correct, as the macro generates them.
    dao.create_users()?.step()?;
    dao.create_addresses()?.step()?;
    dao.create_categories()?.step()?;
    dao.create_products()?.step()?;
    dao.create_orders()?.step()?;
    dao.create_order_items()?.step()?;

    dao.insert_users()?.step()?;
    dao.insert_addresses()?.step()?;
    dao.insert_categories()?.step()?;
    dao.insert_products()?.step()?;
    dao.insert_orders()?.step()?;
    dao.insert_order_items()?.step()?;

    // ---------------------------------------------------------
    // TEST 1: Simple Reuse (No Bindings)
    // ---------------------------------------------------------
    println!("\n--- TEST 1: Caching Reuse (Running q_low_stock 3 times) ---");
    for i in 1..=3 {
        println!("Run #{}", i);
        let stmt = dao.q_low_stock()?;
        for row in stmt.query(LowStock) {
            let r = row?;
            println!("  Found: {} (Stock: {})", r.name, r.stock);
        }
    }

    // ---------------------------------------------------------
    // TEST 2: Parameter Reuse
    // ---------------------------------------------------------
    println!("\n--- TEST 2: Parameter Changing (q_item_count) ---");

    {
        println!("Case A (Count > 0):");
        let stmt = dao.q_item_count()?;
        stmt.bind_parameter(1, 0)?;
        for row in stmt.query(OrdersItemCount) {
            println!("  {:?}", row?);
        }
    }

    {
        println!("Case B (Count > 50 - Should be empty):");
        let stmt = dao.q_item_count()?;
        stmt.bind_parameter(1, 50)?;
        for row in stmt.query(OrdersItemCount) {
            println!("  {:?}", row?);
        }
    }

    {
        println!("Case C (Count > 1):");
        let stmt = dao.q_item_count()?;
        stmt.bind_parameter(1, 1)?;
        for row in stmt.query(OrdersItemCount) {
            println!("  {:?}", row?);
        }
    }

    // ---------------------------------------------------------
    // TEST 3: Stress Loop
    // ---------------------------------------------------------
    println!("\n--- TEST 3: Stress Loop (100,000 iterations) ---");
    let bench_start = Instant::now();

    for _ in 0..100_000 {
        let stmt = dao.q_complex_join()?;
        let count = stmt.query(Test1).count();
        assert_eq!(count, 3);
    }

    println!(
        "100,000 queries finished in {:.2?}",
        bench_start.elapsed()
    );

    let total_elapsed = now.elapsed();
    println!("\nTotal Elapsed: {:.2?}", total_elapsed);

    Ok(())
}