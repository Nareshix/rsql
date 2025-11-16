use libsqlite3_sys::{
    SQLITE_OK, sqlite3_clear_bindings, sqlite3_finalize, sqlite3_reset, sqlite3_step,
    sqlite3_stmt,
};

use crate::{
    internal_sqlite::row::Rows,
    traits::{ row_mapper::RowMapper, to_sql::ToSql},
    utility::{error::SqliteFailure, utils::get_sqlite_failiure},
};

use crate::{internal_sqlite::connection::Connection};

#[allow(dead_code)]
// #[derive(Debug)]
pub struct Statement<'a> {
    pub conn: &'a Connection,
    pub stmt: *mut sqlite3_stmt,
}

impl Drop for Statement<'_> {
    fn drop(&mut self) {
        unsafe { sqlite3_finalize(self.stmt) };
    }
}

impl Statement<'_> {
    //TODO
    //If any of the sqlite3_bind_*() routines are called with a NULL pointer for the
    // prepared statement or with a prepared statement for which sqlite3_step()
    // has been called more recently than sqlite3_reset(), then the call will return SQLITE_MISUSE.
    // If any sqlite3_bind_() routine is passed a prepared statement that has been finalized,
    // the result is undefined and probably harmful.

    ///note index start from 1 and not 0
    /// TODO consider &impl ToSql to prevent moving?
    #[allow(unused)]
    pub fn bind_parameter(&self, index: i32, value: impl ToSql) -> Result<(), SqliteFailure> {
        let code = unsafe { value.bind_to(self.stmt, index) };

        if code != SQLITE_OK {
            let (code, error_msg) = unsafe { get_sqlite_failiure(self.conn.db) };
            Err(SqliteFailure { code, error_msg })
        } else {
            Ok(())
        }
    }

    #[allow(unused)]
    pub fn reset(&self) {
        //TODO error hanndle code
        unsafe { sqlite3_reset(self.stmt) };
    }
    #[allow(unused)]
    pub fn clear_bindings(&self) {
        unsafe { sqlite3_clear_bindings(self.stmt) };
        //  return code is always SQLITE_OK according to rusqlite
        // Not necessary in ur case, pls double cehck again TODO
    }

    /// returns SQLITE_ROW  if there is available rows
    /// else, SQLITE_DONE (or smth else TODO check again)
    ///
    /// TODO: do we need to warn whether returns nothing? like during compile time check
    pub fn step(&self) -> i32 {
        // TODO error handling?
        unsafe { sqlite3_step(self.stmt) }
    }

    pub fn query<'a, M: RowMapper>(&'a self, mapper: M) -> Rows<'a, M> {
        Rows { stmt: self, mapper }
    }
}
