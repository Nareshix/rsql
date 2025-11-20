use libsqlite3_sys::{
    self as ffi, SQLITE_OK, SQLITE_OPEN_CREATE, SQLITE_OPEN_MEMORY, SQLITE_OPEN_READWRITE, sqlite3,
    sqlite3_busy_timeout, sqlite3_finalize, sqlite3_stmt,
};
use std::{
    cell::RefCell,
    collections::HashMap,
    ffi::{CString, c_int},
    ptr,
};

use crate::{
    errors::connection::{SqliteOpenErrors, SqlitePrepareErrors},
    internal_sqlite::statement::Statement,
    utility::utils::{close_db, get_sqlite_failiure},
};

// defaults to true cuz we would want to immediately use it after preparation
// only becomes false when SQLITE_DONE or (<Statement> or <Rows>)  gets dropped
pub(crate) struct RawStatement {
    pub(crate) stmt: *mut sqlite3_stmt,
    pub(crate) in_use: bool,
}

impl Drop for RawStatement {
    fn drop(&mut self) {
        unsafe {
            sqlite3_finalize(self.stmt);
        }
    }
}
pub struct Connection {
    pub(crate) db: *mut sqlite3,
    pub(crate) cache: RefCell<HashMap<String, RawStatement>>, //ik
}

impl Drop for Connection {
    fn drop(&mut self) {
        unsafe {
            self.cache.get_mut().clear();
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
            CString::new(filename).map_err(|_| SqliteOpenErrors::EmbeddedNullInFileName {
                filename: filename.to_owned(),
            })?;

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
            let cache = RefCell::new(HashMap::new());
            Ok(Connection { db, cache })
        } else {
            let (code, error_msg) = unsafe { get_sqlite_failiure(db) };
            unsafe {
                close_db(db);
            };
            Err(SqliteOpenErrors::SqliteFailure { code, error_msg })
        }
    }

    pub fn prepare(&self, sql: &str) -> Result<Statement<'_>, SqlitePrepareErrors> {
        let mut cache = self.cache.borrow_mut();
        // cache exists and isnt being in used (the stmt has alr been reseted and bindings are cleared)
        if let Some(raw_stmt) = cache.get_mut(sql)
            && !raw_stmt.in_use
        {
            let stmt = raw_stmt.stmt;
            raw_stmt.in_use = true;
            return Ok(Statement { conn: self, stmt, key: Some(sql.to_string())});
        }

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
        if code != SQLITE_OK {
            let (code, error_msg) = unsafe { get_sqlite_failiure(self.db) };
            return Err(SqlitePrepareErrors::SqliteFailure { code, error_msg });
        }

        // cache exists but is being used. Do not cache it
        if cache.contains_key(sql) {
            Ok(Statement { conn: self, stmt, key:None })
        }
        // cache do not exist
        else {
            cache.insert(sql.to_string(), RawStatement { stmt, in_use: true });
            Ok(Statement { conn: self, stmt, key:Some(sql.to_string()) })
        }
    }
}
