use std::ffi::CStr;

use libsqlite3_sys::sqlite3_stmt;

// no errors cuz sqlite does implicit conversion
pub trait FromSql {
    /// # Safety
    /// The caller must ensure that `stmt` points to a valid, stepped statement
    /// that is currently on a row (i.e., sqlite3_step has returned SQLITE_ROW).
    unsafe fn from_col(stmt: *mut sqlite3_stmt, index: i32) -> Self;
}

impl FromSql for String {
    unsafe fn from_col(stmt: *mut sqlite3_stmt, index: i32) -> Self {
        let c_string = unsafe { libsqlite3_sys::sqlite3_column_text(stmt, index) } as *const i8;
        unsafe { CStr::from_ptr(c_string).to_string_lossy().into_owned() }
    }
}

// TODO 
// impl FromSql for &str {
//     unsafe fn from_col(stmt: *mut sqlite3_stmt, index: i32) -> Self {
//         let c_string = unsafe { libsqlite3_sys::sqlite3_column_text(stmt, index) } as *const i8;
//         unsafe { CStr::from_ptr(c_string).to_string_lossy().into_owned() }
//     }
// }

impl FromSql for f64 {
    unsafe fn from_col(stmt: *mut sqlite3_stmt, index: i32) -> Self {
        unsafe { libsqlite3_sys::sqlite3_column_double(stmt, index) }
    }
}

impl FromSql for i32 {
    unsafe fn from_col(stmt: *mut sqlite3_stmt, index: i32) -> Self {
        unsafe { libsqlite3_sys::sqlite3_column_int(stmt, index) }
    }
}
impl FromSql for i64 {
    unsafe fn from_col(stmt: *mut sqlite3_stmt, index: i32) -> Self {
        unsafe { libsqlite3_sys::sqlite3_column_int64(stmt, index) }
    }
}

//TODO

// impl<T:FromSql> FromSql for Option<T>{
//     unsafe fn from_col(stmt: *mut sqlite3_stmt, index: i32) -> Self {
//         if
//     }
// }
