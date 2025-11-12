use libsqlite3_sys::{
    self as ffi, SQLITE_OK, SQLITE_OPEN_CREATE, SQLITE_OPEN_MEMORY, SQLITE_OPEN_READWRITE, sqlite3,
};
use std::{
    ffi::{CString, c_int},
    ptr,
};

use crate::utility::{error::Error, utils::get_sqlite_failiure};
use crate::{internal_sqlite::statement::Statement, utility::utils::close_db};

//TODO use Refcell for better ergonomics (UNconfirmed)
pub struct Connection {
    pub(crate) db: *mut sqlite3,
}

impl Drop for Connection {
    fn drop(&mut self) {
        unsafe {
            close_db(self.db);
        };
    }
}

impl Connection {
    pub fn open(filename: &str) -> Result<Self, Error> {
        let flag = SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE;

        Connection::open_with_flags(filename, flag)
    }

    pub fn open_memory() -> Result<Self, Error> {
        let flag = SQLITE_OPEN_MEMORY | SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE
;
        Connection::open_with_flags(":memory:", flag)
    }

    // The flags refer to what mode to open the db in (readwrite, memory, etc)
    fn open_with_flags(filename: &str, flag: c_int) -> Result<Self, Error> {
        let mut db = ptr::null_mut();
        //TODO handle expect
        let c_filename = CString::new(filename).expect("CString::new failed");

        let code = unsafe { ffi::sqlite3_open_v2(c_filename.as_ptr(), &mut db, flag, ptr::null()) };

        if code == SQLITE_OK && db.is_null() {
            unsafe {
                close_db(db);
            };
            Err(Error::ConnectionAllocationFailed)
        } else if code == SQLITE_OK {
            Ok(Connection { db })
        } else {
            let (code, error_msg) = unsafe { get_sqlite_failiure(db) };
            unsafe {
                close_db(db);
            };
            Err(Error::SqliteFailure { code, error_msg })
        }
    }

    pub fn prepare(&self, sql: &str) -> Result<Statement<'_>, Error> {
        //TODO handle expect
        let c_sql_query = CString::new(sql).expect("CString::new failed");
        let mut stmt = ptr::null_mut();
        let code = unsafe {
            ffi::sqlite3_prepare_v2(
                self.db,
                c_sql_query.as_ptr(),
                -1,
                &mut stmt,
                ptr::null_mut(),
            )
        };

        // TODO        
        // *ppStmt is left pointing to a compiled prepared statement that can be executed
        //  using sqlite3_step(). If there is an error, *ppStmt is set to NULL. 
        // If the input text contains no SQL (if the input is an empty string or a comment)
    //  then *ppStmt is set to NULL. The calling procedure is responsible for deleting 
        // the compiled SQL statement using sqlite3_finalize() after it has finished with it. 
        // ppStmt may not be NULL.
        if code == ffi::SQLITE_OK {
            Ok(Statement { conn: self, stmt })
        } else {
            let (code, error_msg) = unsafe { get_sqlite_failiure(self.db) };
            Err(Error::SqliteFailure { code, error_msg })
        }
    }
}
