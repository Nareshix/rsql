
use std::time::Instant;
use rsql::{Connection, LazyStmt, SqlMapping, lazy_sql}; // Adjust imports based on your lib.rs exports

// -----------------------------------------------------------------------------
// 1. Result Structs (Same as original)
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
// 2. The DAO Definition using the Macro
// -----------------------------------------------------------------------------

// Assuming the attribute macro is named `lazy_sql` or matches the name you exported
#[lazy_sql] 
pub struct ShopDao<'a> {
    // REQUIRED: The macro hardcodes access to `self.db.db`
    db: &'a Connection,

    // --- DDL ---
    #[sql("CREATE TABLE users (
        user_id       INTEGER PRIMARY KEY AUTOINCREMENT,
        username      TEXT NOT NULL UNIQUE,
        email         TEXT NOT NULL UNIQUE,
        created_at    TIMESTAMP DEFAULT CURRENT_TIMESTAMP
    );")]
    create_users: LazyStmt,

    #[sql("CREATE TABLE addresses (
        address_id    INTEGER PRIMARY KEY AUTOINCREMENT,
        user_id       INTEGER NOT NULL,
        address_line  TEXT NOT NULL,
        city          TEXT NOT NULL,
        country       TEXT NOT NULL,
        is_primary    BOOLEAN DEFAULT 0,
        FOREIGN KEY (user_id) REFERENCES users(user_id)
    );")]
    create_addresses: LazyStmt,

    #[sql("CREATE TABLE categories (
        category_id   INTEGER PRIMARY KEY AUTOINCREMENT,
        name          TEXT NOT NULL,
        parent_id     INTEGER,
        FOREIGN KEY (parent_id) REFERENCES categories(category_id)
    );")]
    create_categories: LazyStmt,

    #[sql("CREATE TABLE products (
        product_id    INTEGER PRIMARY KEY AUTOINCREMENT,
        category_id   INTEGER NOT NULL,
        name          TEXT NOT NULL,
        price         REAL NOT NULL CHECK(price > 0),
        stock         INTEGER NOT NULL DEFAULT 0,
        FOREIGN KEY (category_id) REFERENCES categories(category_id)
    );")]
    create_products: LazyStmt,

    #[sql("CREATE TABLE orders (
        order_id      INTEGER PRIMARY KEY AUTOINCREMENT,
        user_id       INTEGER NOT NULL,
        order_time    TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        status        TEXT NOT NULL CHECK(status IN ('pending','paid','shipped','cancelled')),
        FOREIGN KEY (user_id) REFERENCES users(user_id)
    );")]
    create_orders: LazyStmt,

    #[sql("CREATE TABLE order_items (
        item_id       INTEGER PRIMARY KEY AUTOINCREMENT,
        order_id      INTEGER NOT NULL,
        product_id    INTEGER NOT NULL,
        quantity      INTEGER NOT NULL CHECK(quantity > 0),
        price_each    REAL NOT NULL CHECK(price_each > 0),
        FOREIGN KEY (order_id) REFERENCES orders(order_id),
        FOREIGN KEY (product_id) REFERENCES products(product_id)
    );")]
    create_order_items: LazyStmt,

    // --- INSERTS ---
    #[sql("INSERT INTO users (username, email) VALUES
        ('alice', 'alice@example.com'),
        ('bob',   'bob@example.com'),
        ('charlie','charlie@example.com');")]
    insert_users: LazyStmt,

    #[sql("INSERT INTO addresses (user_id, address_line, city, country, is_primary) VALUES
        (1, '123 Street A', 'Singapore', 'SG', 1),
        (1, '456 Backup Ave', 'Tokyo', 'JP', 0),
        (2, '789 Main Road', 'London', 'UK', 1);")]
    insert_addresses: LazyStmt,

    #[sql("INSERT INTO categories (name, parent_id) VALUES
        ('Electronics', NULL),
        ('Computers', 1),
        ('Phones', 1),
        ('Accessories', 1);")]
    insert_categories: LazyStmt,

    #[sql("INSERT INTO products (category_id, name, price, stock) VALUES
        (2, 'Laptop Pro 15',   2500, 10),
        (2, 'Gaming PC X',     3200, 5),
        (3, 'Smartphone Z',    999,  25),
        (4, 'USB-C Cable',     15,   200);")]
    insert_products: LazyStmt,

    #[sql("INSERT INTO orders (user_id, status) VALUES
        (1, 'pending'),
        (1, 'paid'),
        (2, 'shipped');")]
    insert_orders: LazyStmt,

    #[sql("INSERT INTO order_items (order_id, product_id, quantity, price_each) VALUES
        (1, 1, 1, 2500),
        (1, 4, 3, 15),
        (2, 3, 1, 999),
        (3, 4, 2, 15);")]
    insert_order_items: LazyStmt,

    // --- UPDATES / DELETES ---
    #[sql("UPDATE products SET price = price * 1.10 WHERE product_id = 1;")]
    update_price: LazyStmt,

    #[sql("UPDATE orders SET status = 'shipped' WHERE order_id = 1;")]
    update_order_shipped: LazyStmt,

    #[sql("UPDATE products SET stock = stock + 50 WHERE product_id = 4;")]
    update_stock: LazyStmt,

    #[sql("DELETE FROM addresses WHERE address_id = 2;")]
    delete_address: LazyStmt,

    #[sql("DELETE FROM orders WHERE status = 'cancelled';")]
    delete_cancelled: LazyStmt,

    // --- QUERIES ---
    
    // 1. Complex Join
    #[sql("SELECT 
        o.order_id,
        u.username,
        SUM(oi.quantity * oi.price_each) AS total_amount,
        o.status
    FROM orders o
    JOIN users u ON o.user_id = u.user_id
    JOIN order_items oi ON oi.order_id = o.order_id
    GROUP BY o.order_id;")]
    q_complex_join: LazyStmt,

    // 2. Low Stock
    #[sql("SELECT name, stock FROM products WHERE stock < 20;")]
    q_low_stock: LazyStmt,

    // 3. Hierarchy (Self Join)
    #[sql("SELECT 
        c1.name AS category,
        c2.name AS parent_category
    FROM categories c1
    LEFT JOIN categories c2 ON c1.parent_id = c2.category_id;")]
    q_categories: LazyStmt,

    // 4. Primary Address
    #[sql("SELECT 
        u.username,
        a.address_line,
        a.city,
        a.country
    FROM users u
    JOIN addresses a ON u.user_id = a.user_id
    WHERE a.is_primary = 1;")]
    q_addresses: LazyStmt,

    // 5. Having Clause (With Bindings)
    #[sql("SELECT order_id, COUNT(*) AS num_items
        FROM order_items
        GROUP BY order_id
        HAVING COUNT(*) > ?;")]
    q_item_count: LazyStmt,

    // 6. Complex Filter (With 3 Bindings)
    #[sql("SELECT product_id, name, price, stock, category_id
        FROM products
        WHERE (stock < ?1 AND price > ?2)
        OR category_id = (SELECT category_id FROM categories WHERE name = ?3);")]
    q_product_filter: LazyStmt,
}

// -----------------------------------------------------------------------------
// 3. Main Execution
// -----------------------------------------------------------------------------
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let now = Instant::now();

    // 1. Open Connection
    let conn = Connection::open_memory().expect("Failed to open memory db");

    // 2. Initialize DAO
    let mut dao = ShopDao::new(&conn);

    println!("--- Initializing Schema & Data ---");
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
    // Goal: Confirm we don't re-prepare or crash on subsequent calls
    // ---------------------------------------------------------
    println!("\n--- TEST 1: Caching Reuse (Running q_low_stock 3 times) ---");
    for i in 1..=3 {
        println!("Run #{}", i);
        // The macro checks if stmt is null. 
        // Run #1: It is null -> Prepares it.
        // Run #2: It is NOT null -> Reuses raw pointer.
        let stmt = dao.q_low_stock()?; 
        let xx=  stmt.query(LowStock);
        for row in xx {
            let r = row?;
            println!("  Found: {} (Stock: {})", r.name, r.stock);
        }
        // `stmt` drops here -> calls sqlite3_reset
    }

    // ---------------------------------------------------------
    // TEST 2: Parameter Reuse
    // Goal: Confirm old parameters are cleared and new ones applied
    // ---------------------------------------------------------
    println!("\n--- TEST 2: Parameter Changing (q_item_count) ---");

    // Case A: Count > 0 (Should match multiple orders)
    {
        println!("Case A (Count > 0):");
        let stmt = dao.q_item_count()?;
        stmt.bind_parameter(1, 0)?; 
        for row in stmt.query(OrdersItemCount) {
            println!(" mom {:?}", row?);
        }
    } // stmt drops -> resets & clears bindings

    // Case B: Count > 50 (Should match NOTHING)
    // If 'clear_bindings' failed, this might behave weirdly
    {
        println!("Case B (Count > 50 - Should be empty):");
        let stmt = dao.q_item_count()?;
        stmt.bind_parameter(1, 50)?; 
        for row in stmt.query(OrdersItemCount) {
            println!("  {:?}", row?);
        }
    }

    
    // Case C: Count > 1 (Should match Order #1 only)
    {
        println!("Case C (Count > 1):");
        let stmt = dao.q_item_count()?;
        stmt.bind_parameter(1, 1)?; 
        for row in stmt.query(OrdersItemCount) {
            println!(" {:?}", row?);
        }
    }

    // ---------------------------------------------------------
    // TEST 3: Stress Loop
    // Goal: Ensure rapid reuse doesn't leak memory or segfault
    // ---------------------------------------------------------
    println!("\n--- TEST 3: Stress Loop (1,000 iterations) ---");
    let bench_start = Instant::now();
    
    for _ in 0..100_000 {
        let stmt = dao.q_complex_join()?;
        // Just consume the iterator to force execution, don't print
        let count = stmt.query(Test1).count(); 
        assert_eq!(count, 3); // Verify logic holds up every time
    }
    
    println!("1,000,000 queries finished in {:.2?}", bench_start.elapsed());

    let total_elapsed = now.elapsed();
    println!("\nTotal Elapsed: {:.2?}", total_elapsed);

    Ok(())
}