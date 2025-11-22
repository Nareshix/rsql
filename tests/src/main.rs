use std::time::Instant;
use rsql::{Connection, LazyStmt, SqlMapping, lazy_sql};
use rusqlite::{params, Connection as RusqliteConnection};

// =============================================================================
// 1. RSQL SETUP (The Lazy DAO)
// =============================================================================

#[lazy_sql]
pub struct ShopDao<'a> {
    db: &'a Connection,

    // DDL
    #[sql("CREATE TABLE products (product_id INTEGER PRIMARY KEY, stock INTEGER);")]
    create_products: LazyStmt,
    #[sql("CREATE TABLE orders (order_id INTEGER PRIMARY KEY AUTOINCREMENT, user_id INTEGER);")]
    create_orders: LazyStmt,
    #[sql("CREATE TABLE order_items (item_id INTEGER PRIMARY KEY, order_id INTEGER, product_id INTEGER);")]
    create_items: LazyStmt,

    // Queries
    #[sql("INSERT INTO products (stock) VALUES (10000);")]
    init_product: LazyStmt,

    #[sql("INSERT INTO orders (user_id) VALUES (?);")]
    insert_order: LazyStmt,

    #[sql("INSERT INTO order_items (order_id, product_id) VALUES (?, ?);")]
    insert_item: LazyStmt,

    #[sql("UPDATE products SET stock = stock - 1 WHERE product_id = ?;")]
    deduct_stock: LazyStmt,
}

fn bench_rsql(orders_to_process: i32) {
    let conn = Connection::open_memory().unwrap();
    let mut dao = ShopDao::new(&conn);

    // Setup
    dao.create_products().unwrap().step().unwrap();
    dao.create_orders().unwrap().step().unwrap();
    dao.create_items().unwrap().step().unwrap();
    
    // Create 5 products
    for _ in 0..5 { dao.init_product().unwrap().step().unwrap(); }

    let start = Instant::now();

    for _ in 0..orders_to_process {
        // 1. Insert Order (User ID 1)
        // SCOPE 1: Created, executed, and DROPPED here
        {
            let mut stmt = dao.insert_order().unwrap();
            stmt.bind_parameter(1, 1).unwrap();
            stmt.step().unwrap();
        } // <--- stmt drops here, calling sqlite3_reset()

        let order_id = 1; 

        // 2. Add Item A (Product 1)
        // SCOPE 2
        {
            let mut stmt = dao.insert_item().unwrap();
            stmt.bind_parameter(1, order_id).unwrap();
            stmt.bind_parameter(2, 1).unwrap();
            stmt.step().unwrap();
        } // <--- Resets 'insert_item'

        // 3. Deduct Stock A
        // SCOPE 3
        {
            let mut stmt = dao.deduct_stock().unwrap();
            stmt.bind_parameter(1, 1).unwrap();
            stmt.step().unwrap();
        }

        // 4. Add Item B (Product 2)
        // SCOPE 4: Now safe to reuse 'insert_item' because Scope 2 reset it!
        {
            let mut stmt = dao.insert_item().unwrap();
            stmt.bind_parameter(1, order_id).unwrap();
            stmt.bind_parameter(2, 2).unwrap();
            stmt.step().unwrap();
        }

        // 5. Deduct Stock B
        // SCOPE 5
        {
            let mut stmt = dao.deduct_stock().unwrap();
            stmt.bind_parameter(1, 2).unwrap();
            stmt.step().unwrap();
        }
    }

    println!("RSQL (Lazy DAO)         : {:.2?}", start.elapsed());
}
// =============================================================================
// 2. RUSQLITE (NO CACHE)
// =============================================================================

fn bench_rusqlite_no_cache(orders_to_process: i32) {
    let conn = RusqliteConnection::open_in_memory().unwrap();

    conn.execute_batch("
        CREATE TABLE products (product_id INTEGER PRIMARY KEY, stock INTEGER);
        CREATE TABLE orders (order_id INTEGER PRIMARY KEY AUTOINCREMENT, user_id INTEGER);
        CREATE TABLE order_items (item_id INTEGER PRIMARY KEY, order_id INTEGER, product_id INTEGER);
    ").unwrap();

    for _ in 0..5 { conn.execute("INSERT INTO products (stock) VALUES (10000)", []).unwrap(); }

    let start = Instant::now();

    for _ in 0..orders_to_process {
        // 1. Insert Order
        // prepare() compiles the SQL string EVERY TIME
        conn.prepare("INSERT INTO orders (user_id) VALUES (?)").unwrap()
            .execute(params![1]).unwrap();

        let order_id = 1;

        // 2. Add Item A
        conn.prepare("INSERT INTO order_items (order_id, product_id) VALUES (?, ?)").unwrap()
            .execute(params![order_id, 1]).unwrap();

        // 3. Deduct Stock A
        conn.prepare("UPDATE products SET stock = stock - 1 WHERE product_id = ?").unwrap()
            .execute(params![1]).unwrap();

        // 4. Add Item B
        conn.prepare("INSERT INTO order_items (order_id, product_id) VALUES (?, ?)").unwrap()
            .execute(params![order_id, 2]).unwrap();

        // 5. Deduct Stock B
        conn.prepare("UPDATE products SET stock = stock - 1 WHERE product_id = ?").unwrap()
            .execute(params![2]).unwrap();
    }

    println!("Rusqlite (No Cache)     : {:.2?}", start.elapsed());
}

// =============================================================================
// 3. RUSQLITE (WITH CACHE)
// =============================================================================

fn bench_rusqlite_cached(orders_to_process: i32) {
    let conn = RusqliteConnection::open_in_memory().unwrap();

    conn.execute_batch("
        CREATE TABLE products (product_id INTEGER PRIMARY KEY, stock INTEGER);
        CREATE TABLE orders (order_id INTEGER PRIMARY KEY AUTOINCREMENT, user_id INTEGER);
        CREATE TABLE order_items (item_id INTEGER PRIMARY KEY, order_id INTEGER, product_id INTEGER);
    ").unwrap();

    for _ in 0..5 { conn.execute("INSERT INTO products (stock) VALUES (10000)", []).unwrap(); }

    let start = Instant::now();

    for _ in 0..orders_to_process {
        // 1. Insert Order
        // prepare_cached() uses HashMap lookup + LRU Cache
        conn.prepare_cached("INSERT INTO orders (user_id) VALUES (?)").unwrap()
            .execute(params![1]).unwrap();

        let order_id = 1;

        // 2. Add Item A
        conn.prepare_cached("INSERT INTO order_items (order_id, product_id) VALUES (?, ?)").unwrap()
            .execute(params![order_id, 1]).unwrap();

        // 3. Deduct Stock A
        conn.prepare_cached("UPDATE products SET stock = stock - 1 WHERE product_id = ?").unwrap()
            .execute(params![1]).unwrap();

        // 4. Add Item B
        conn.prepare_cached("INSERT INTO order_items (order_id, product_id) VALUES (?, ?)").unwrap()
            .execute(params![order_id, 2]).unwrap();

        // 5. Deduct Stock B
        conn.prepare_cached("UPDATE products SET stock = stock - 1 WHERE product_id = ?").unwrap()
            .execute(params![2]).unwrap();
    }

    println!("Rusqlite (Cached)       : {:.2?}", start.elapsed());
}

// =============================================================================
// RUNNER
// =============================================================================

fn main() {
    let orders = 5_000;
    println!("--- BENCHMARKING REAL WORLD TRANSACTION ({} Orders) ---", orders);
    println!("Total Queries: {} (5 per order)\n", orders * 5);

    bench_rsql(orders);
    bench_rusqlite_no_cache(orders);
    bench_rusqlite_cached(orders);
}