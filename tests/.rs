#![feature(prelude_import)]
#[macro_use]
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use rsql::{
    SqlMapping,
    internal_sqlite::efficient::{
        lazy_connection::LazyConnection, lazy_statement::LazyStmt,
    },
    lazy_sql,
};
use std::time::Instant;
struct Test1 {
    order_id: i32,
    username: String,
    sum: f64,
    status: String,
}
#[automatically_derived]
impl ::core::fmt::Debug for Test1 {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field4_finish(
            f,
            "Test1",
            "order_id",
            &self.order_id,
            "username",
            &self.username,
            "sum",
            &self.sum,
            "status",
            &&self.status,
        )
    }
}
struct Test1Mapper;
impl rsql::traits::row_mapper::RowMapper for Test1Mapper {
    type Output = Test1;
    unsafe fn map_row(&self, stmt: *mut libsqlite3_sys::sqlite3_stmt) -> Self::Output {
        let order_id = unsafe {
            <i32 as rsql::traits::from_sql::FromSql>::from_sql(stmt, 0i32)
        };
        let username = unsafe {
            <String as rsql::traits::from_sql::FromSql>::from_sql(stmt, 1i32)
        };
        let sum = unsafe {
            <f64 as rsql::traits::from_sql::FromSql>::from_sql(stmt, 2i32)
        };
        let status = unsafe {
            <String as rsql::traits::from_sql::FromSql>::from_sql(stmt, 3i32)
        };
        Self::Output {
            order_id,
            username,
            sum,
            status,
        }
    }
}
#[allow(non_upper_case_globals)]
const Test1: Test1Mapper = Test1Mapper;
struct LowStock {
    name: String,
    stock: i32,
}
#[automatically_derived]
impl ::core::fmt::Debug for LowStock {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field2_finish(
            f,
            "LowStock",
            "name",
            &self.name,
            "stock",
            &&self.stock,
        )
    }
}
struct LowStockMapper;
impl rsql::traits::row_mapper::RowMapper for LowStockMapper {
    type Output = LowStock;
    unsafe fn map_row(&self, stmt: *mut libsqlite3_sys::sqlite3_stmt) -> Self::Output {
        let name = unsafe {
            <String as rsql::traits::from_sql::FromSql>::from_sql(stmt, 0i32)
        };
        let stock = unsafe {
            <i32 as rsql::traits::from_sql::FromSql>::from_sql(stmt, 1i32)
        };
        Self::Output { name, stock }
    }
}
#[allow(non_upper_case_globals)]
const LowStock: LowStockMapper = LowStockMapper;
struct OrdersItemCount {
    order_id: i32,
    num_items: i32,
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
        let order_id = unsafe {
            <i32 as rsql::traits::from_sql::FromSql>::from_sql(stmt, 0i32)
        };
        let num_items = unsafe {
            <i32 as rsql::traits::from_sql::FromSql>::from_sql(stmt, 1i32)
        };
        Self::Output {
            order_id,
            num_items,
        }
    }
}
#[allow(non_upper_case_globals)]
const OrdersItemCount: OrdersItemCountMapper = OrdersItemCountMapper;
struct ProductFilter {
    product_id: i32,
    name: String,
    price: f64,
    stock: i32,
    category_id: i32,
}
#[automatically_derived]
impl ::core::fmt::Debug for ProductFilter {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field5_finish(
            f,
            "ProductFilter",
            "product_id",
            &self.product_id,
            "name",
            &self.name,
            "price",
            &self.price,
            "stock",
            &self.stock,
            "category_id",
            &&self.category_id,
        )
    }
}
struct ProductFilterMapper;
impl rsql::traits::row_mapper::RowMapper for ProductFilterMapper {
    type Output = ProductFilter;
    unsafe fn map_row(&self, stmt: *mut libsqlite3_sys::sqlite3_stmt) -> Self::Output {
        let product_id = unsafe {
            <i32 as rsql::traits::from_sql::FromSql>::from_sql(stmt, 0i32)
        };
        let name = unsafe {
            <String as rsql::traits::from_sql::FromSql>::from_sql(stmt, 1i32)
        };
        let price = unsafe {
            <f64 as rsql::traits::from_sql::FromSql>::from_sql(stmt, 2i32)
        };
        let stock = unsafe {
            <i32 as rsql::traits::from_sql::FromSql>::from_sql(stmt, 3i32)
        };
        let category_id = unsafe {
            <i32 as rsql::traits::from_sql::FromSql>::from_sql(stmt, 4i32)
        };
        Self::Output {
            product_id,
            name,
            price,
            stock,
            category_id,
        }
    }
}
#[allow(non_upper_case_globals)]
const ProductFilter: ProductFilterMapper = ProductFilterMapper;
pub struct ShopDao<'a> {
    db: &'a LazyConnection,
    q_complex_join: LazyStmt,
    q_low_stock: LazyStmt,
    q_categories: LazyStmt,
    q_addresses: LazyStmt,
    q_item_count: LazyStmt,
    q_product_filter: LazyStmt,
}
impl<'a> ShopDao<'a> {
    pub fn new(db: &'a LazyConnection) -> Self {
        Self {
            db: db,
            q_complex_join: LazyStmt {
                sql_query: "SELECT \n        o.order_id,\n        u.username,\n        SUM(oi.quantity * oi.price_each) AS total_amount,\n        o.status\n        FROM orders o\n        JOIN users u ON o.user_id = u.user_id\n        JOIN order_items oi ON oi.order_id = o.order_id\n        GROUP BY o.order_id;",
                stmt: std::ptr::null_mut(),
            },
            q_low_stock: LazyStmt {
                sql_query: "SELECT name, stock FROM products WHERE stock < 20;",
                stmt: std::ptr::null_mut(),
            },
            q_categories: LazyStmt {
                sql_query: "SELECT \n        c1.name AS category,\n        c2.name AS parent_category\n    FROM categories c1\n    LEFT JOIN categories c2 ON c1.parent_id = c2.category_id;",
                stmt: std::ptr::null_mut(),
            },
            q_addresses: LazyStmt {
                sql_query: "SELECT \n        u.username,\n        a.address_line,\n        a.city,\n        a.country\n    FROM users u\n    JOIN addresses a ON u.user_id = a.user_id\n    WHERE a.is_primary = 1;",
                stmt: std::ptr::null_mut(),
            },
            q_item_count: LazyStmt {
                sql_query: "SELECT order_id, COUNT(*) AS num_items\n        FROM order_items\n        GROUP BY order_id\n        HAVING COUNT(*) > ?;",
                stmt: std::ptr::null_mut(),
            },
            q_product_filter: LazyStmt {
                sql_query: "SELECT product_id, name, price, stock, category_id\n        FROM products\n        WHERE (stock < ?1 AND price > ?2)\n        OR category_id = (SELECT category_id FROM categories WHERE name = ?3);",
                stmt: std::ptr::null_mut(),
            },
        }
    }
    pub fn q_complex_join(
        &mut self,
    ) -> Result<
        rsql::internal_sqlite::efficient::preparred_statement::PreparredStmt,
        rsql::errors::SqliteFailure,
    > {
        if self.q_complex_join.stmt.is_null() {
            let c_sql = std::ffi::CString::new(self.q_complex_join.sql_query).unwrap();
            let code = unsafe {
                libsqlite3_sys::sqlite3_prepare_v2(
                    self.db.db,
                    c_sql.as_ptr(),
                    -1,
                    &mut self.q_complex_join.stmt,
                    std::ptr::null_mut(),
                )
            };
            if code != libsqlite3_sys::SQLITE_OK {
                let (code, error_msg) = unsafe {
                    rsql::utility::utils::get_sqlite_failiure(self.db.db)
                };
                return Err(rsql::errors::SqliteFailure {
                    code,
                    error_msg,
                });
            }
        }
        Ok(rsql::internal_sqlite::efficient::preparred_statement::PreparredStmt {
            stmt: self.q_complex_join.stmt,
            conn: self.db.db,
        })
    }
    pub fn q_low_stock(
        &mut self,
    ) -> Result<
        rsql::internal_sqlite::efficient::preparred_statement::PreparredStmt,
        rsql::errors::SqliteFailure,
    > {
        if self.q_low_stock.stmt.is_null() {
            let c_sql = std::ffi::CString::new(self.q_low_stock.sql_query).unwrap();
            let code = unsafe {
                libsqlite3_sys::sqlite3_prepare_v2(
                    self.db.db,
                    c_sql.as_ptr(),
                    -1,
                    &mut self.q_low_stock.stmt,
                    std::ptr::null_mut(),
                )
            };
            if code != libsqlite3_sys::SQLITE_OK {
                let (code, error_msg) = unsafe {
                    rsql::utility::utils::get_sqlite_failiure(self.db.db)
                };
                return Err(rsql::errors::SqliteFailure {
                    code,
                    error_msg,
                });
            }
        }
        Ok(rsql::internal_sqlite::efficient::preparred_statement::PreparredStmt {
            stmt: self.q_low_stock.stmt,
            conn: self.db.db,
        })
    }
    pub fn q_categories(
        &mut self,
    ) -> Result<
        rsql::internal_sqlite::efficient::preparred_statement::PreparredStmt,
        rsql::errors::SqliteFailure,
    > {
        if self.q_categories.stmt.is_null() {
            let c_sql = std::ffi::CString::new(self.q_categories.sql_query).unwrap();
            let code = unsafe {
                libsqlite3_sys::sqlite3_prepare_v2(
                    self.db.db,
                    c_sql.as_ptr(),
                    -1,
                    &mut self.q_categories.stmt,
                    std::ptr::null_mut(),
                )
            };
            if code != libsqlite3_sys::SQLITE_OK {
                let (code, error_msg) = unsafe {
                    rsql::utility::utils::get_sqlite_failiure(self.db.db)
                };
                return Err(rsql::errors::SqliteFailure {
                    code,
                    error_msg,
                });
            }
        }
        Ok(rsql::internal_sqlite::efficient::preparred_statement::PreparredStmt {
            stmt: self.q_categories.stmt,
            conn: self.db.db,
        })
    }
    pub fn q_addresses(
        &mut self,
    ) -> Result<
        rsql::internal_sqlite::efficient::preparred_statement::PreparredStmt,
        rsql::errors::SqliteFailure,
    > {
        if self.q_addresses.stmt.is_null() {
            let c_sql = std::ffi::CString::new(self.q_addresses.sql_query).unwrap();
            let code = unsafe {
                libsqlite3_sys::sqlite3_prepare_v2(
                    self.db.db,
                    c_sql.as_ptr(),
                    -1,
                    &mut self.q_addresses.stmt,
                    std::ptr::null_mut(),
                )
            };
            if code != libsqlite3_sys::SQLITE_OK {
                let (code, error_msg) = unsafe {
                    rsql::utility::utils::get_sqlite_failiure(self.db.db)
                };
                return Err(rsql::errors::SqliteFailure {
                    code,
                    error_msg,
                });
            }
        }
        Ok(rsql::internal_sqlite::efficient::preparred_statement::PreparredStmt {
            stmt: self.q_addresses.stmt,
            conn: self.db.db,
        })
    }
    pub fn q_item_count(
        &mut self,
    
    ) -> Result<
        rsql::internal_sqlite::efficient::preparred_statement::PreparredStmt,
        rsql::errors::SqliteFailure,
    > {
        if self.q_item_count.stmt.is_null() {
            let c_sql = std::ffi::CString::new(self.q_item_count.sql_query).unwrap();
            let code = unsafe {
                libsqlite3_sys::sqlite3_prepare_v2(
                    self.db.db,
                    c_sql.as_ptr(),
                    -1,
                    &mut self.q_item_count.stmt,
                    std::ptr::null_mut(),
                )
            };
            if code != libsqlite3_sys::SQLITE_OK {
                let (code, error_msg) = unsafe {
                    rsql::utility::utils::get_sqlite_failiure(self.db.db)
                };
                return Err(rsql::errors::SqliteFailure {
                    code,
                    error_msg,
                });
            }
        }
        Ok(rsql::internal_sqlite::efficient::preparred_statement::PreparredStmt {
            stmt: self.q_item_count.stmt,
            conn: self.db.db,
        })
    }
    pub fn q_product_filter(
        &mut self,
    ) -> Result<
        rsql::internal_sqlite::efficient::preparred_statement::PreparredStmt,
        rsql::errors::SqliteFailure,
    > {
        if self.q_product_filter.stmt.is_null() {
            let c_sql = std::ffi::CString::new(self.q_product_filter.sql_query).unwrap();
            let code = unsafe {
                libsqlite3_sys::sqlite3_prepare_v2(
                    self.db.db,
                    c_sql.as_ptr(),
                    -1,
                    &mut self.q_product_filter.stmt,
                    std::ptr::null_mut(),
                )
            };
            if code != libsqlite3_sys::SQLITE_OK {
                let (code, error_msg) = unsafe {
                    rsql::utility::utils::get_sqlite_failiure(self.db.db)
                };
                return Err(rsql::errors::SqliteFailure {
                    code,
                    error_msg,
                });
            }
        }
        Ok(rsql::internal_sqlite::efficient::preparred_statement::PreparredStmt {
            stmt: self.q_product_filter.stmt,
            conn: self.db.db,
        })
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let now = Instant::now();
    let conn = LazyConnection::open("oi.db").expect("Failed to open memory db");
    let mut dao = ShopDao::new(&conn);
    for i in 1..=3 {
        {
            ::std::io::_print(format_args!("Run #{0}\n", i));
        };
        let stmt = dao.q_low_stock()?;
        for row in stmt.query(LowStock) {
            let r = row?;
            {
                ::std::io::_print(
                    format_args!("  Found: {0} (Stock: {1})\n", r.name, r.stock),
                );
            };
        }
    }
    {
        let stmt = dao.q_item_count()?;
        stmt.bind_parameter(1, 0)?;
        for row in stmt.query(OrdersItemCount) {
            {
                ::std::io::_print(format_args!("  {0:?}\n", row?));
            };
        }
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
    let bench_start = Instant::now();
    for _ in 0..100_000 {
        let stmt = dao.q_complex_join()?;
        let count = stmt.query(Test1).count();
        match (&count, &3) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
    }
    {
        ::std::io::_print(
            format_args!(
                "1,000,000 queries finished in {0:.2?}\n",
                bench_start.elapsed(),
            ),
        );
    };
    let total_elapsed = now.elapsed();
    {
        ::std::io::_print(format_args!("\nTotal Elapsed: {0:.2?}\n", total_elapsed));
    };
    Ok(())
}
