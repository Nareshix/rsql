#![feature(prelude_import)]
#[macro_use]
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use rsql::{Connection, LazyStmt, lazy_sql, utility::utils::prepare_stmt};
pub struct UserDao<'a> {
    db: &'a Connection,
    get_by_id_stmt: LazyStmt,
    insert_stmt: LazyStmt,
}
impl<'a> UserDao<'a> {
    pub fn new(db: &'a Connection) -> Self {
        Self {
            db,
            get_by_id_stmt: LazyStmt {
                sql_query: "SELECT * FROM users WHERE id = ?",
                stmt: std::ptr::null_mut(),
            },
            insert_stmt: LazyStmt {
                sql_query: "",
                stmt: std::ptr::null_mut(),
            },
        }
    }
    pub fn get_by_id_stmt(
        &mut self,
    ) -> Result<&mut LazyStmt, rsql::errors::connection::SqlitePrepareErrors> {
        if self.get_by_id_stmt.stmt.is_null() {
            unsafe {
                prepare_stmt(
                    self.db.db,
                    &mut self.get_by_id_stmt.stmt,
                    self.get_by_id_stmt.sql_query,
                )?;
            }
        }
        Ok(&mut self.get_by_id_stmt)
    }
    pub fn insert_stmt(
        &mut self,
    ) -> Result<&mut LazyStmt, rsql::errors::connection::SqlitePrepareErrors> {
        if self.insert_stmt.stmt.is_null() {
            unsafe {
                prepare_stmt(
                    self.db.db,
                    &mut self.insert_stmt.stmt,
                    self.insert_stmt.sql_query,
                )?;
            }
        }
        Ok(&mut self.insert_stmt)
    }
}
fn main() {
    let conn = Connection::open_memory().unwrap();
    let mut dao = UserDao::new(&conn);
    let stmt = dao.get_by_id_stmt();
    {
        ::std::io::_print(format_args!("Dao created!\n"));
    };
}
