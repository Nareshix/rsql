use libsqlite3_sys::{
    self as ffi, SQLITE_OK, SQLITE_OPEN_CREATE, SQLITE_OPEN_MEMORY, SQLITE_OPEN_READWRITE, sqlite3, sqlite3_busy_timeout,
};
use std::{
    ffi::{CString, c_int},
    ptr,
};

use crate::{
    errors::connection::{SqliteOpenErrors, SqlitePrepareErrors},
    internal_sqlite::statement::Statement,
    utility::utils::{close_db, get_sqlite_failiure},
};

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

        let c_filename =
            CString::new(filename).map_err(|_| SqliteOpenErrors::EmbeddedNullInFileName)?;

        let code = unsafe { ffi::sqlite3_open_v2(c_filename.as_ptr(), &mut db, flag, ptr::null()) };

        if code == SQLITE_OK && db.is_null() {
            unsafe {
                close_db(db);
            };
            Err(SqliteOpenErrors::ConnectionAllocationFailed)
        } else if code == SQLITE_OK {
            //TODO sqlite3_busy_timeout does return an int. It is nearly a gurantee for this 
            // function to never fail. but its still good to handle it. 
            unsafe { sqlite3_busy_timeout(db, 5000) };
            Ok(Connection { db })
        } else {
            let (code, error_msg) = unsafe { get_sqlite_failiure(db) };
            unsafe {
                close_db(db);
            };
            Err(SqliteOpenErrors::SqliteFailure { code, error_msg })
        }
    }

    pub fn prepare(&self, sql: &str) -> Result<Statement<'_>, SqlitePrepareErrors> {
        let c_sql_query = CString::new(sql).map_err(|_| SqlitePrepareErrors::EmbeddedNullInSql)?;

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
            Err(SqlitePrepareErrors::SqliteFailure { code, error_msg })
        }
    }
}
