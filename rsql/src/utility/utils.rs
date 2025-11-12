use libsqlite3_sys::{
    self as ffi, SQLITE_BUSY, SQLITE_FLOAT, SQLITE_INTEGER, SQLITE_NULL, SQLITE_TEXT, sqlite3,
    sqlite3_errcode,
};
use std::{
    ffi::{CStr, c_char},
    thread,
    time::Duration,
};

use crate::utility::error::Error;

pub enum RustTypes {
    Integer,
    String,
    Float,
    Null,
}

pub fn sqlite_to_rust_type_mapping(sqlite_type: i32) -> Result<RustTypes, Error> {
    match sqlite_type {
        SQLITE_INTEGER => Ok(RustTypes::Integer),
        SQLITE_FLOAT => Ok(RustTypes::Float),
        SQLITE_TEXT => Ok(RustTypes::String),
        SQLITE_NULL => Ok(RustTypes::Null),
        _ => Err(Error::SqliteToRustConversionFailiure),
    }
}

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

/// This fn also handles SQLITE_BUSY error code, allowing for graceful shutdown
///
/// # Safety
///
/// - db must be a valid sqlite3 connection which is not NULL  
pub unsafe fn close_db(db: *mut sqlite3) {
    //TODO idt u should loop? perhaps a timeout? porblematic during long running services tho...
    loop {
        let code = unsafe { ffi::sqlite3_close(db) };

        if code == SQLITE_BUSY {
            thread::sleep(Duration::from_millis(5000));
        } else {
            break;
        }
    }
}
