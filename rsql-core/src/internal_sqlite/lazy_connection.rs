// TODO dont use to_string, just use &str when creating Statemnet. but low prio
use libsqlite3_sys::{
    self as ffi, SQLITE_OK, SQLITE_OPEN_CREATE, SQLITE_OPEN_MEMORY, SQLITE_OPEN_READWRITE, sqlite3,
    sqlite3_busy_timeout,
};
use std::{
    ffi::{CString, c_int},
    ptr,
};

use crate::errors::connection::SqliteOpenErrors;
use crate::utility::utils::{close_db, get_sqlite_failiure};

pub struct Connection {
    pub db: *mut sqlite3,
}

impl Drop for Connection {
    fn drop(&mut self) {
        unsafe {
            close_db(self.db);
        };
    }
}

impl Connection {
    pub fn open(filename: &str) -> Result<Self, SqliteOpenErrors> {
        let flag = SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE;

        Connection::open_with_flags(filename, flag)
    }

    pub fn open_memory() -> Result<Self, SqliteOpenErrors> {
        let flag = SQLITE_OPEN_MEMORY | SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE;
        Connection::open_with_flags(":memory:", flag)
    }

    // The flags refer to what mode to open the db in (readwrite, memory, etc)
    fn open_with_flags(filename: &str, flag: c_int) -> Result<Self, SqliteOpenErrors> {
        let mut db = ptr::null_mut();

        let c_filename = CString::new(filename).unwrap(); //TODO

        let code = unsafe { ffi::sqlite3_open_v2(c_filename.as_ptr(), &mut db, flag, ptr::null()) };

        if code == SQLITE_OK && db.is_null() {
            unsafe {
                close_db(db);
            };
            Err(SqliteOpenErrors::ConnectionAllocationFailed)
        } else if code == SQLITE_OK {
            //TODO sqlite3_busy_timeout does return an int. It is nearly a gurantee for this
            // function to never fail. but its still good to handle it. If it fails mean 
            // the sql query is taking more than 5 second which means its inefficent lol
            unsafe { sqlite3_busy_timeout(db, 5000) };
            Ok(Self { db })
        } else {
            let (code, error_msg) = unsafe { get_sqlite_failiure(db) };
            unsafe {
                close_db(db);
            };
            Err(SqliteOpenErrors::SqliteFailure { code, error_msg })
        }
    }
}
