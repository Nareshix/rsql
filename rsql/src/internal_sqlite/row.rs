use std::ffi::CStr;

use libsqlite3_sys::{SQLITE_DONE, SQLITE_ROW, sqlite3_column_int, sqlite3_column_int64, sqlite3_column_text, sqlite3_step, sqlite3_stmt};

use crate::{internal_sqlite::statement::Statement, traits::from_sql::{FromSql, RowMapper}};

//TODO example only, eventually make it dynamic
#[allow(dead_code)]
#[derive(Debug)]
pub struct Row {
    pub id: i64,
    pub username: String,
    pub email: String,
}



#[allow(dead_code)]
pub struct Rows<'a, M: RowMapper> {
    pub stmt: &'a Statement<'a>,
    pub mapper: M, // It holds the mapper that will be used.
}

impl<'a, M: RowMapper> Iterator for Rows<'a, M> {
    // The Item is the `Output` type associated with our mapper `M`.
    // The compiler knows this is `Person` when we pass a `PersonMapper`.
    type Item = M::Output;

    fn next(&mut self) -> Option<Self::Item> {
        let result_code = unsafe { sqlite3_step(self.stmt.stmt) };

        if result_code == SQLITE_ROW {
            // Call the map_row method on our stored mapper instance.
            let item = unsafe { self.mapper.map_row(self.stmt.stmt) };
            Some(item)
        } else {
            // SQLITE_DONE or an error occurred.
            None
        }
    }
}