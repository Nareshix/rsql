use libsqlite3_sys as ffi;
use std::{
    ffi::{CStr, CString, c_int},
    ptr,
};

pub struct Connection {
    pub handle: *mut ffi::sqlite3,
}

impl Drop for Connection {
    fn drop(&mut self) {
        unsafe {
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
                //TODO the rror message isnt rlly helpful, so tryto imporve on it
                Err("Sqlite is unable to allocate memory to hold the conn object".to_string())
            }
        } else if code == ffi::SQLITE_OK {
            Ok(Connection { handle: db_handle })
        } else {
            unsafe {
                // sqlite says its not necessary to  to drop c_error_msg
                let c_error_msg = ffi::sqlite3_errmsg(db_handle);
                let safe_error_msg = CStr::from_ptr(c_error_msg);
                let error_msg = safe_error_msg.to_string_lossy().into_owned();

                ffi::sqlite3_close(db_handle);
                Err(error_msg)
            }
        }
    }
}
