use libsqlite3_sys as ffi;
use std::{
    ffi::{CStr, CString, c_int},
    ptr,
};

use crate::sqlite_operations::utils::{ detailed_error_msg};

pub struct Connection {
    pub handle: *mut ffi::sqlite3,
}

impl Drop for Connection {
    fn drop(&mut self) {
        unsafe {
            // TODO this returns  a code, and handle if it fails (SQLITE_BUSY)
            ffi::sqlite3_close(self.handle);
        }
    }
}



#[allow(dead_code)]
impl Connection {
    pub fn open(filename: &str) -> Result<Connection, String> {
        let flag = ffi::SQLITE_OPEN_READWRITE | ffi::SQLITE_OPEN_CREATE;

        Connection::open_with_flags(filename, flag)
    }

    /// - The flags refer to what mode to open the db in (readwrite, memory, etc)
    ///  - Returns a successful sqlite connection to do ur operations on (prepare, execute, etc). Error msg is returned as a String.
    fn open_with_flags(filename: &str, flags: c_int) -> Result<Connection, String> {
        let mut db_handle = ptr::null_mut();

        let c_filename = CString::new(filename).expect("CString::new failed");

        let code = unsafe {
            ffi::sqlite3_open_v2(c_filename.as_ptr(), &mut db_handle, flags, ptr::null())
        };
        
        if code == ffi::SQLITE_OK && db_handle.is_null() {
            unsafe {
                ffi::sqlite3_close(db_handle);
                //TODO the error message alone may not be suffice,  so tryto imporve on it.
                // return the errorr msg from the status code as well to be specific
                // as there is no errors from db
                let c_error_msg = ffi::sqlite3_errstr(code);
                let safe_c_error_msg = CStr::from_ptr(c_error_msg);
                let error_msg = safe_c_error_msg.to_string_lossy().into_owned();

                let full_error_msg = format!(
                    "Sqlite is unable to allocate memory to hold the conn object
                    Error code: {}
                    Error message:{} ",
                    code, error_msg
                );

                Err(full_error_msg)
            }
        } else if code == ffi::SQLITE_OK {
            Ok(Connection { handle: db_handle })
        } else {
            unsafe {
                let error_msg = detailed_error_msg(db_handle);
                ffi::sqlite3_close(db_handle);
                Err(error_msg)
            }
        }
    }
}
