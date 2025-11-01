use libsqlite3_sys::{
    self as ffi, SQLITE_OK, sqlite3_clear_bindings, sqlite3_finalize, sqlite3_reset, sqlite3_stmt,
};

use crate::{connection::Connection, error::Error, utils::get_sqlite_error_msg};
use std::{
    ffi::{CString, c_char},
    ptr,
};

#[allow(unused)]
enum BindTypes<'a> {
    Double(f64),
    Int(i64),
    Null,
    Text(&'a str),
    // Blob()  //TODO
}

#[allow(unused)]
fn str_for_sqlite(
    s: &[u8],
) -> (
    *const c_char,
    ffi::sqlite3_uint64,
    ffi::sqlite3_destructor_type,
) {
    let len = s.len();
    let (ptr, dtor_info) = if len != 0 {
        (s.as_ptr().cast::<c_char>(), ffi::SQLITE_TRANSIENT())
    } else {
        // Return a pointer guaranteed to live forever
        ("".as_ptr().cast::<c_char>(), ffi::SQLITE_STATIC())
    };
    (ptr, len as ffi::sqlite3_uint64, dtor_info)
}

#[allow(dead_code)]
pub struct Statement<'a> {
    conn: &'a Connection,
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
    //TODO private for now, cuz focus on auto caching later in future might open it
    fn prepare(&'_ self, sql: &str) -> Result<Statement<'_>, Error> {
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
    fn bind_parameter(&self, index: i32, value: BindTypes) -> Result<(), Error> {
        let code = match value {
            BindTypes::Null => unsafe { ffi::sqlite3_bind_null(self.stmt, index) },
            BindTypes::Double(value) => unsafe {
                ffi::sqlite3_bind_double(self.stmt, index, value)
            },
            BindTypes::Int(value) => unsafe { ffi::sqlite3_bind_int64(self.stmt, index, value) },
            BindTypes::Text(value) => unsafe {
                let (c_str, len, destructor) = str_for_sqlite(value.as_bytes());
                ffi::sqlite3_bind_text64(
                    self.stmt,
                    index,
                    c_str,
                    len,
                    destructor,
                    ffi::SQLITE_UTF8 as _,
                )
            },
        };

        if code != SQLITE_OK {
            let error_msg = unsafe { get_sqlite_error_msg(self.conn.get_db()) };
            Err(Error::SqliteFailiure(error_msg))
        } else {
            Ok(())
        }
    }

    fn reset(&self) {
        //TODO error hanndle code
        let code = unsafe { sqlite3_reset(self.stmt) };
    }

    fn clear_bindings(&self) {
         unsafe { sqlite3_clear_bindings(self.stmt) };
        //  return code is always SQLITE_OK according to rusqlite
        // Not necessary in ur case, pls double cehck again TODO
    }
}
