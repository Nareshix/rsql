use libsqlite3_sys::{self as ffi, SQLITE_BUSY, sqlite3};
use std::{
    ffi::{CStr,},
     thread, time::Duration,
};


/// # Safety
///
/// - db must be a valid sqlite3 connection which is not NULL  
pub unsafe fn get_sqlite_error_msg(db: *mut sqlite3) -> String {
    let safe_error_msg: &CStr = unsafe {
        // sqlite internally handles dropping c_error_msg
        let c_error_msg = ffi::sqlite3_errmsg(db);
        CStr::from_ptr(c_error_msg)
    };

    safe_error_msg.to_string_lossy().into_owned()
}

/// # Safety
///
/// - db must be a valid sqlite3 connection which is not NULL  
pub unsafe fn close_db(db: *mut sqlite3) {
    loop {
        let code = unsafe { ffi::sqlite3_close(db) };

        if code == SQLITE_BUSY {
            thread::sleep(Duration::from_millis(5000));
        } else {
            break;
        }
    }
}
