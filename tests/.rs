#![feature(prelude_import)]
#[macro_use]
extern crate std;
use rsql::{SqlMapping, internal_sqlite::efficient::lazy_connection::LazyConnection, lazy_sql};
#[prelude_import]
use std::prelude::rust_2024::*;
struct OrdersItemCount {
    pub order_id: i32,
    pub num_items: i32,
}
#[automatically_derived]
impl ::core::fmt::Debug for OrdersItemCount {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field2_finish(
            f,
            "OrdersItemCount",
            "order_id",
            &self.order_id,
            "num_items",
            &&self.num_items,
        )
    }
}
struct OrdersItemCountMapper;
impl rsql::traits::row_mapper::RowMapper for OrdersItemCountMapper {
    type Output = OrdersItemCount;
    unsafe fn map_row(&self, stmt: *mut libsqlite3_sys::sqlite3_stmt) -> Self::Output {
        let order_id = unsafe { <i32 as rsql::traits::from_sql::FromSql>::from_sql(stmt, 0i32) };
        let num_items = unsafe { <i32 as rsql::traits::from_sql::FromSql>::from_sql(stmt, 1i32) };
        Self::Output {
            order_id,
            num_items,
        }
    }
}
#[allow(non_upper_case_globals)]
const OrdersItemCount: OrdersItemCountMapper = OrdersItemCountMapper;
pub struct ShopDao<'a> {
    __db: &'a rsql::internal_sqlite::efficient::lazy_connection::LazyConnection,
    q_complex_join: rsql::internal_sqlite::efficient::lazy_statement::LazyStmt,
    q_low_stock: rsql::internal_sqlite::efficient::lazy_statement::LazyStmt,
    q_item_count: rsql::internal_sqlite::efficient::lazy_statement::LazyStmt,
}
impl<'a> ShopDao<'a> {
    pub fn new(db: &'a rsql::internal_sqlite::efficient::lazy_connection::LazyConnection) -> Self {
        Self {
            __db: db,
            q_complex_join: rsql::internal_sqlite::efficient::lazy_statement::LazyStmt {
                sql_query: "SELECT
        o.order_id,
        u.username,
        SUM(oi.quantity * oi.price_each) AS total_amount,
        o.status
        FROM orders o
        JOIN users u ON o.user_id = u.user_id
        JOIN order_items oi ON oi.order_id = o.order_id
        GROUP BY o.order_id;",
                stmt: std::ptr::null_mut(),
            },
            q_low_stock: rsql::internal_sqlite::efficient::lazy_statement::LazyStmt {
                sql_query: "SELECT name FROM products WHERE stock < 20",
                stmt: std::ptr::null_mut(),
            },
            q_item_count: rsql::internal_sqlite::efficient::lazy_statement::LazyStmt {
                sql_query: "SELECT order_id, COUNT(*) AS num_items
        FROM order_items
        GROUP BY order_id
        HAVING COUNT(*) > ?;",
                stmt: std::ptr::null_mut(),
            },
        }
    }
    /** **SQL**
    ```sql
    SELECT
        o.order_id,
        u.username,
        SUM(oi.quantity * oi.price_each) AS total_amount,
        o.status
    FROM
        orders o
        JOIN users u ON o.user_id = u.user_id
        JOIN order_items oi ON oi.order_id = o.order_id
    GROUP BY
        o.order_id;*/
    pub fn q_complex_join(
        &mut self,
    ) -> Result<
        rsql::internal_sqlite::efficient::preparred_statement::PreparredStmt,
        rsql::errors::connection::SqlitePrepareErrors,
    > {
        if self.q_complex_join.stmt.is_null() {
            unsafe {
                rsql::utility::utils::prepare_stmt(
                    self.__db.db,
                    &mut self.q_complex_join.stmt,
                    self.q_complex_join.sql_query,
                )?;
            }
        }
        Ok(
            rsql::internal_sqlite::efficient::preparred_statement::PreparredStmt {
                stmt: self.q_complex_join.stmt,
                conn: self.__db.db,
            },
        )
    }
    /** **SQL**
    ```sql
    SELECT
        name
    FROM
        products
    WHERE
        stock < 20*/
    pub fn q_low_stock(
        &mut self,
    ) -> Result<
        rsql::internal_sqlite::efficient::preparred_statement::PreparredStmt,
        rsql::errors::connection::SqlitePrepareErrors,
    > {
        if self.q_low_stock.stmt.is_null() {
            unsafe {
                rsql::utility::utils::prepare_stmt(
                    self.__db.db,
                    &mut self.q_low_stock.stmt,
                    self.q_low_stock.sql_query,
                )?;
            }
        }
        Ok(
            rsql::internal_sqlite::efficient::preparred_statement::PreparredStmt {
                stmt: self.q_low_stock.stmt,
                conn: self.__db.db,
            },
        )
    }
    /** **SQL**
    ```sql
    SELECT
        order_id,
        COUNT(*) AS num_items
    FROM
        order_items
    GROUP BY
        order_id
    HAVING
        COUNT(*) > ?;*/
    pub fn q_item_count(
        &mut self,
    ) -> Result<
        rsql::internal_sqlite::efficient::preparred_statement::PreparredStmt,
        rsql::errors::connection::SqlitePrepareErrors,
    > {
        if self.q_item_count.stmt.is_null() {
            unsafe {
                rsql::utility::utils::prepare_stmt(
                    self.__db.db,
                    &mut self.q_item_count.stmt,
                    self.q_item_count.sql_query,
                )?;
            }
        }
        Ok(
            rsql::internal_sqlite::efficient::preparred_statement::PreparredStmt {
                stmt: self.q_item_count.stmt,
                conn: self.__db.db,
            },
        )
    }
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
        {
            ::std::io::_print(format_args!("{0}\n", i?.num_items));
        };
    }
    {
        let stmt = dao.q_item_count()?;
        stmt.bind_parameter(1, 50)?;
        for row in stmt.query(OrdersItemCount) {
            {
                ::std::io::_print(format_args!("  {0:?}\n", row?));
            };
        }
    }
    {
        {
            ::std::io::_print(format_args!("Case C (Count > 1):\n"));
        };
        let stmt = dao.q_item_count()?;
        stmt.bind_parameter(1, 1)?;
        for row in stmt.query(OrdersItemCount) {
            {
                ::std::io::_print(format_args!("  {0:?}\n", row?));
            };
        }
    }
    Ok(())
}
