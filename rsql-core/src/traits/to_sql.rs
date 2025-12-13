use std::ffi::CString;

use libsqlite3_sys::{self as ffi, SQLITE_TRANSIENT, sqlite3_stmt};

// TODO is there a way to avoid conversion? or is it unecessary? test the speed later

// BLOB not implemented yet. TODO
pub trait ToSql {
    /// - it is ok for it to be self consuming (tho it only applies to String)
    ///   because we are not gonna be using this rust type anymore
    /// - Indexes start at 1
    ///# Safety
    ///
    /// Pass in the stmt pointer (not the address). it uses sqlite3_bind_* to bind it to the correct type
    unsafe fn bind_to(self, stmt: *mut sqlite3_stmt, index: i32) -> i32;
}

impl ToSql for String {
    unsafe fn bind_to(self, stmt: *mut sqlite3_stmt, index: i32) -> i32 {
        let c_str = CString::new(self).unwrap(); //TODO
        unsafe { ffi::sqlite3_bind_text(stmt, index, c_str.as_ptr(), -1, SQLITE_TRANSIENT()) }
    }
}

impl ToSql for &str {
    unsafe fn bind_to(self, stmt: *mut sqlite3_stmt, index: i32) -> i32 {
        let c_str = CString::new(self).unwrap(); //TODO
        unsafe { ffi::sqlite3_bind_text(stmt, index, c_str.as_ptr(), -1, SQLITE_TRANSIENT()) }
    }
}

impl ToSql for i32 {
    unsafe fn bind_to(self, stmt: *mut sqlite3_stmt, index: i32) -> i32 {
        unsafe { ffi::sqlite3_bind_int(stmt, index, self) }
    }
}

impl ToSql for i64 {
    unsafe fn bind_to(self, stmt: *mut sqlite3_stmt, index: i32) -> i32 {
        unsafe { ffi::sqlite3_bind_int64(stmt, index, self) }
    }
}

impl ToSql for f64 {
    unsafe fn bind_to(self, stmt: *mut sqlite3_stmt, index: i32) -> i32 {
        unsafe { ffi::sqlite3_bind_double(stmt, index, self) }
    }
}

impl<T: ToSql> ToSql for Option<T> {
    unsafe fn bind_to(self, stmt: *mut sqlite3_stmt, index: i32) -> i32 {
        match self {
            Some(rust_value) => unsafe { rust_value.bind_to(stmt, index) },
            None => unsafe { ffi::sqlite3_bind_null(stmt, index) },
        }
    }
}
