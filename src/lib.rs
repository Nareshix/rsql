use libsqlite3_sys::{self as ffi, SQLITE_OK, SQLITE_OPEN_CREATE, SQLITE_OPEN_READWRITE};
use std::{
    ffi::{CString, c_int},
    ptr,
};

use crate::{
    utils::{close_db, get_sqlite_error_msg},
    error::Error,
};

pub mod utils;
pub mod error;
// pub mod prepare;

pub struct Connection {
    db: *mut ffi::sqlite3,
}

impl Drop for Connection {
    fn drop(&mut self) {
        // TODO handle SQL_BUSY
        unsafe { ffi::sqlite3_close(self.db) };
    }
}

impl Connection {
    pub fn open(filename: &str) -> Result<Connection, Error> {
        let flag = SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE;

        Connection::open_with_flags(filename, flag)
    }

    // The flags refer to what mode to open the db in (readwrite, memory, etc)
    fn open_with_flags(filename: &str, flags: c_int) -> Result<Connection, Error> {
        let mut db = ptr::null_mut();

        let c_filename = CString::new(filename).expect("CString::new failed");

        let code =
            unsafe { ffi::sqlite3_open_v2(c_filename.as_ptr(), &mut db, flags, ptr::null()) };

        if code == SQLITE_OK && db.is_null() {
            unsafe {
                close_db(db);
            }

            Err(Error::OpenFailure(
                "SQLite is unable to allocate memory to hold the sqlite3 object".to_string(),
            ))
        } else if code == SQLITE_OK {
            Ok(Connection { db })
        } else {
            let error_msg = unsafe { get_sqlite_error_msg(db) };
            Err(Error::SqliteFailiure(error_msg))
        }
    }
}
