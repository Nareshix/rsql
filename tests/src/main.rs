use rsql::{
    SqlMapping,
    internal_sqlite::efficient::{lazy_connection::LazyConnection, lazy_statement::LazyStmt},
    lazy_sql,
};

#[derive(Debug, SqlMapping)]
struct OrdersItemCount {
    pub order_id: i32,
    pub num_items: i32,
}

#[lazy_sql]
pub struct ShopDao {
    // 1. Complex Join
    #[sql(
        "SELECT 
        o.order_id,
        u.username,
        SUM(oi.quantity * oi.price_each) AS total_amount,
        o.status
        FROM orders o
        JOIN users u ON o.user_id = u.user_id
        JOIN order_items oi ON oi.order_id = o.order_id
        GROUP BY o.order_id;"
    )]
    q_complex_join: LazyStmt,

    // 2. Low Stock
    #[sql("SELECT name, stock FROM products WHERE stock < 20;")]
    q_low_stock: LazyStmt,

    // 3. Hierarchy (Self Join)
    #[sql(
        "SELECT 
        c1.name AS category,
        c2.name AS parent_category
    FROM categories c1
    LEFT JOIN categories c2 ON c1.parent_id = c2.category_id;"
    )]
    q_categories: LazyStmt,

    // 4. Primary Address
    #[sql(
        "SELECT 
        u.username,
        a.address_line,
        a.city,
        a.country
    FROM users u
    JOIN addresses a ON u.user_id = a.user_id
    WHERE a.is_primary = 1;"
    )]
    q_addresses: LazyStmt,

    // 5. Having Clause (With Bindings)
    #[sql(
        "SELECT order_id, COUNT(*) AS num_items
        FROM order_items
        GROUP BY order_id
        HAVING COUNT(*) > ?;"
    )]
    q_item_count: LazyStmt,

    // 6. Complex Filter (With 3 Bindings)
    #[sql(
        "SELECT product_id, name, price, stock, category_id
        FROM products
        WHERE (stock < ?1 AND price > ?2)
        OR category_id = (SELECT category_id FROM categories WHERE name = ?3);"
    )]
    q_product_filter: LazyStmt,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = LazyConnection::open("oi.db").unwrap();

    let mut dao = ShopDao::new(&conn);

    // dao.create_addresses()?.step()?;

    let x = {
        let stmt = dao.q_item_count()?;
        stmt.bind_parameter(1, 0)?;
        stmt.query(OrdersItemCount)
    };
    for i in x {
        println!("{:?}", i?);
    }
    {
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

    Ok(())
}
