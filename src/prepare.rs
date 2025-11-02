use libsqlite3_sys::{
    self as ffi, SQLITE_OK, sqlite3_clear_bindings, sqlite3_finalize, sqlite3_reset, sqlite3_step, sqlite3_stmt
};

use crate::{connection::Connection, error::Error, to_sql::ToSql, utils::get_sqlite_error_msg};
use std::{
    ffi::{CString},
    ptr,
};


#[allow(dead_code)]
pub struct Statement<'conn> {
    conn: &'conn Connection,
    stmt: *mut sqlite3_stmt,
}

impl Drop for Statement<'_> {
    fn drop(&mut self) {
        //handle case when it fials to finalise TODO
        unsafe { sqlite3_finalize(self.stmt) };
    }
}

#[allow(dead_code)]
impl Connection {
    pub fn prepare(&self, sql: &str) -> Result<Statement<'_>, Error> {
        let c_sql_query = CString::new(sql).expect("CString::new failed");
        let mut stmt = ptr::null_mut();
        let code = unsafe {
            ffi::sqlite3_prepare_v2(
                self.get_db(),
                c_sql_query.as_ptr(),
                -1,
                &mut stmt,
                ptr::null_mut(),
            )
        };

        if code == ffi::SQLITE_OK {
            Ok(Statement { conn: self, stmt })
        } else {
            let error_msg = unsafe { get_sqlite_error_msg(self.get_db()) };
            Err(Error::SqliteFailiure(error_msg))
        }
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
    #[allow(unused)]
    pub fn bind_parameter<T: ToSql>(&self, index: i32, value: T) -> Result<(), Error> {
        let code = unsafe{value.bind_to(self.stmt, index)};
        
        if code != SQLITE_OK {
            let error_msg = unsafe { get_sqlite_error_msg(self.conn.get_db()) };
            Err(Error::SqliteFailiure(error_msg))
        } else {
            Ok(())
        }
    }

    fn reset(&self) {
        //TODO error hanndle code
        unsafe { sqlite3_reset(self.stmt) };
    }

    fn clear_bindings(&self) {
         unsafe { sqlite3_clear_bindings(self.stmt) };
        //  return code is always SQLITE_OK according to rusqlite
        // Not necessary in ur case, pls double cehck again TODO
    }

    pub fn step(&self){
        //TODO doesnt support SELECT (duh)
        unsafe {sqlite3_step(self.stmt)};
    }
}
