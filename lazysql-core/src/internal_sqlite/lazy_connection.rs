use libsqlite3_sys::{
    self as ffi, SQLITE_OK, SQLITE_OPEN_CREATE, SQLITE_OPEN_MEMORY, SQLITE_OPEN_READWRITE, sqlite3,
    sqlite3_busy_timeout, sqlite3_column_count, sqlite3_column_name, sqlite3_exec,
};
use std::{
    ffi::{CStr, CString, c_int},
    ptr,
};

use crate::{
    errors::connection::SqlitePrepareErrors,
    utility::utils::{close_db, get_sqlite_failiure},
};
use crate::{
    errors::{SqliteFailure, connection::SqliteOpenErrors},
    internal_sqlite::dynamic_rows::DynamicRows,
    utility::utils::prepare_stmt,
};

pub struct LazyConnection {
    pub db: *mut sqlite3,
}

impl Drop for LazyConnection {
    fn drop(&mut self) {
        unsafe {
            close_db(self.db);
        };
    }
}

impl LazyConnection {
    pub fn open(filename: &str) -> Result<Self, SqliteOpenErrors> {
        let flag = SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE;
        LazyConnection::open_with_flags(filename, flag)
    }

    pub fn open_memory() -> Result<Self, SqliteOpenErrors> {
        let flag = SQLITE_OPEN_MEMORY | SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE;
        LazyConnection::open_with_flags(":memory:", flag)
    }

    fn open_with_flags(filename: &str, flag: c_int) -> Result<Self, SqliteOpenErrors> {
        let mut db = ptr::null_mut();
        let c_filename = CString::new(filename).unwrap(); //TODO

        let code = unsafe { ffi::sqlite3_open_v2(c_filename.as_ptr(), &mut db, flag, ptr::null()) };

        if code == SQLITE_OK && db.is_null() {
            unsafe { close_db(db) };
            Err(SqliteOpenErrors::ConnectionAllocationFailed)
        } else if code == SQLITE_OK {
            //TODO sqlite3_busy_timeout does return an int. It is nearly a gurantee for this
            // function to never fail. but its still good to handle it. If it fails mean
            // the sql query is taking more than 5 second which means its inefficent lol

            unsafe { sqlite3_busy_timeout(db, 5000) };
            Ok(Self { db })
        } else {
            let (code, error_msg) = unsafe { get_sqlite_failiure(db) };
            unsafe { close_db(db) };
            Err(SqliteOpenErrors::SqliteFailure { code, error_msg })
        }
    }

    pub fn exec(&self, sql: &str) -> Result<(), SqliteFailure> {
        let c_sql = CString::new(sql).unwrap(); //TODO
        let code = unsafe {
            sqlite3_exec(
                self.db,
                c_sql.as_ptr(),
                None,
                ptr::null_mut(),
                ptr::null_mut(),
            )
        };

        if code != SQLITE_OK {
            let (code, error_msg) = unsafe { get_sqlite_failiure(self.db) };
            return Err(SqliteFailure { code, error_msg });
        }
        Ok(())
    }

    pub fn query_dynamic(&self, sql: &str) -> Result<DynamicRows, SqliteFailure> {
        let mut stmt = std::ptr::null_mut();
        unsafe {
            prepare_stmt(self.db, &mut stmt, sql).map_err(|e| match e {
                SqlitePrepareErrors::SqliteFailure { code, error_msg } => {
                    SqliteFailure { code, error_msg }
                }
            })?;

            let count = sqlite3_column_count(stmt);
            let mut column_names = Vec::with_capacity(count as usize);
            for i in 0..count {
                let ptr = sqlite3_column_name(stmt, i);
                let name = CStr::from_ptr(ptr).to_string_lossy().into_owned();
                column_names.push(name);
            }

            Ok(DynamicRows::new(stmt, column_names))
        }
    }
}
