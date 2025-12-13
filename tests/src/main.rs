use rsql::{
    internal_sqlite::efficient::lazy_connection::LazyConnection,
    lazy_sql,
};

#[lazy_sql("tests/oi.db")]
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

    q_low_stock: sql!("SELECT name  AS mom FROM products WHERE stock < 20"),

    q_item_count: sql!(
        "INSERT INTO USERS(USER_ID) VALUES(?)"
    ),

}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = LazyConnection::open("oi.db").unwrap();
    let mut dao = ShopDao::new(&conn);

    let x= dao.q_low_stock()?;
    for i in x {
        let xx = i?;
        println!("{}", xx.mom);


    }

    // let x = {
    //     let stmt = dao.q_item_count()?;
    //     stmt.bind_parameter(1, 0)?;
    //     stmt.query(OrdersItemCount)
    // };
    // for i in x {
    //     println!("{}", i?.num_items);
    // }
    // {
    //     let stmt = dao.q_item_count()?;
    //     stmt.bind_parameter(1, 50)?;
    //     for row in stmt.query(OrdersItemCount) {
    //         println!("  {:?}", row?);
    //     }
    // }

    // {
    //     println!("Case C (Count > 1):");
    //     let stmt = dao.q_item_count()?;
    //     stmt.bind_parameter(1, 1)?;
    //     for row in stmt.query(OrdersItemCount) {
    //         println!("  {:?}", row?);
    //     }
    // }

    Ok(())
}
