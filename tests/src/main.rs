use rsql::{
    SqlMapping,
    internal_sqlite::efficient::lazy_connection::LazyConnection,
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

    q_low_stock: sql!("SELECT name FROM products WHERE stock < 20"),

    q_item_count: sql!(
        "SELECT order_id, COUNT(*) AS num_items
        FROM order_items
        GROUP BY order_id
        HAVING COUNT(*) > ?;"
    ),

}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = LazyConnection::open("oi.db").unwrap();
    let mut dao = ShopDao::new(&conn);


    let x = {
        let stmt = dao.q_item_count()?;
        stmt.bind_parameter(1, 0)?;
        stmt.query(OrdersItemCount)
    };
    for i in x {
        println!("{}", i?.num_items);
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
