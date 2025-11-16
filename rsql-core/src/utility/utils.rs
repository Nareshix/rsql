use libsqlite3_sys::{
    self as ffi, SQLITE_BUSY, sqlite3,
    sqlite3_errcode,
};
use std::{
    ffi::{CStr, c_char},
    thread,
    time::Duration,
};



pub enum RustTypes {
    Integer,
    String,
    Float,
    Null,
}

// pub fn sqlite_to_rust_type_mapping(sqlite_type: i32) -> Result<RustTypes,SqliteToRustConversionFailiure> {
//     match sqlite_type {
//         SQLITE_INTEGER => Ok(RustTypes::Integer),
//         SQLITE_FLOAT => Ok(RustTypes::Float),
//         SQLITE_TEXT => Ok(RustTypes::String),
//         SQLITE_NULL => Ok(RustTypes::Null),
//         _ => Err(SqliteToRustConversionFailiure),
//     }
// }

/// Internally calls sqlite3_errcode and sqlite3_errmsg to return 
/// specifcally **Error::SqliteFailure** with the necessary code and error_msg
///
///  # Safety
///
/// - db must be a valid sqlite3 connection which is not NULL  
pub unsafe fn get_sqlite_failiure(db: *mut sqlite3) -> (i32, String) {
    let safe_error_msg = unsafe {
        // sqlite internally handles dropping c_error_msg
        let c_error_msg = ffi::sqlite3_errmsg(db);
        CStr::from_ptr(c_error_msg as *const c_char)
    };

    let error_msg = safe_error_msg.to_string_lossy().into_owned();
    let code = unsafe { sqlite3_errcode(db) };

    (code, error_msg)    
}

///
/// # Safety
///
/// - db must be a valid sqlite3 connection which is not NULL  
pub unsafe fn close_db(db: *mut sqlite3) {
    // TODO returns SQLITE_BUSY. but dpeending on the strucurre of my code dont have to deal with it
    // also sqlite3_close_v2 is only for gc languages hence sqlite3_close is preferred
    unsafe { ffi::sqlite3_close(db) };

}
