use libsqlite3_sys::{
    SQLITE_ROW, SQLITE_DONE, SQLITE_INTEGER, SQLITE_FLOAT, SQLITE_TEXT, SQLITE_BLOB, SQLITE_NULL,
    sqlite3_step, sqlite3_column_count, sqlite3_column_name, sqlite3_column_type,
    sqlite3_column_int64, sqlite3_column_double, sqlite3_column_text, sqlite3_finalize
};
use std::ffi::CStr;

use crate::traits::dynamic::Value;

pub struct DynamicRows {
    pub stmt: *mut libsqlite3_sys::sqlite3_stmt,
    pub column_names: Vec<String>,
}

impl Iterator for DynamicRows {
    type Item = Vec<Value>;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if sqlite3_step(self.stmt) == SQLITE_ROW {
                let count = sqlite3_column_count(self.stmt);
                let mut row = Vec::with_capacity(count as usize);

                for i in 0..count {
                    let val = match sqlite3_column_type(self.stmt, i) {
                        SQLITE_INTEGER => Value::Integer(sqlite3_column_int64(self.stmt, i)),
                        SQLITE_FLOAT => Value::Real(sqlite3_column_double(self.stmt, i)),
                        SQLITE_TEXT => {
                            let ptr = sqlite3_column_text(self.stmt, i);
                            let s = CStr::from_ptr(ptr as *const i8).to_string_lossy().into_owned();
                            Value::Text(s)
                        }
                        // TODO SQLITE_BLOB
                        _ => Value::Null,
                    };
                    row.push(val);
                }
                Some(row)
            } else {
                sqlite3_finalize(self.stmt); // Clean up when done
                None
            }
        }
    }
}