use libsqlite3_sys::{self as ffi, sqlite3_stmt};

use crate::{connection::Connection, error::Error, utils::get_sqlite_error_msg};
use std::{
    ffi::{CStr, CString,},
    ptr,
};

#[allow(dead_code)]
pub struct Statement {
    stmt: *mut sqlite3_stmt,
}


#[allow(dead_code)]
impl Connection {
    pub fn prepare(&self, sql: &str) -> Result<Statement, Error> {
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
            Ok(Statement { stmt })
        } else {

            let error_msg = unsafe {
                get_sqlite_error_msg(self.get_db())
            };
            Err(Error::SqliteFailiure(error_msg))

        }
    }
}
