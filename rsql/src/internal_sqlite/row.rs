use std::ffi::CStr;

use libsqlite3_sys::{SQLITE_DONE, sqlite3_column_int64, sqlite3_column_text, sqlite3_step};

use crate::internal_sqlite::statement::Statement;

//TODO example only, eventually make it dynamic
#[allow(dead_code)]
#[derive(Debug)]
pub struct Row {
    pub id: i64,
    pub username: String,
    pub email: String,
}

#[allow(dead_code)]
pub struct Rows<'a> {
    pub stmt: &'a Statement<'a>,
}

impl Iterator for Rows<'_> {
    type Item = Row;

    fn next(&mut self) -> Option<Self::Item> {
        let code = unsafe { sqlite3_step(self.stmt.stmt) };

        if code == SQLITE_DONE {
            return None;
        }

        let id = unsafe { sqlite3_column_int64(self.stmt.stmt, 0) };
        let byte_username = unsafe { sqlite3_column_text(self.stmt.stmt, 1) } as *const i8;

        let username = unsafe { CStr::from_ptr(byte_username).to_string_lossy().into_owned() };

        let c_email = unsafe { sqlite3_column_text(self.stmt.stmt, 2) } as *const i8;

        let email = unsafe { CStr::from_ptr(c_email).to_string_lossy().into_owned() };

        let row = Row {
            id,
            username,
            email,
        };
        Some(row)
    }
}
