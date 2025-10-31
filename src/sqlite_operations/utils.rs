use libsqlite3_sys as ffi;
use std::ffi::CStr;

pub fn detailed_error_msg(db_handle: *mut ffi::sqlite3) -> String{
    unsafe {
        // sqlite says its not necessary to  to drop c_error_msg
        let c_error_msg = ffi::sqlite3_errmsg(db_handle);
        let safe_c_error_msg = CStr::from_ptr(c_error_msg);
        let error_msg = safe_c_error_msg.to_string_lossy().into_owned();

        error_msg
    }
}


// pub fn error_msg_from_status_code() -> String{

// }  

// pub fn close_db_connection(db_handle: *mut ffi::sqlite3) -> Result<(), String> {
//     let code = unsafe {
//         libsqlite3_sys::sqlite3_close()
//     }
// }
