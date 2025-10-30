use libsqlite3_sys as ffi;

use crate::connection::Connection;
use std::{
    ffi::{CStr, CString,},
    ptr,
};

#[allow(dead_code)]
pub struct Statement {
    stmt: *mut ffi::sqlite3_stmt,
}

//maybe dont add drop trait


//*ppStmt is left pointing to a compiled prepared statement that can be executed using sqlite3_step(). 
// If there is an error, *ppStmt is set to NULL. If the input text contains no SQL
// (if the input is an empty string or a comment) then *ppStmt is set to NULL.
// The calling procedure is responsible for deleting the compiled SQL statement 
//using sqlite3_finalize() after it has finished with it. ppStmt may not be NULL.

//Bind any parameters if needed (sqlite3_bind_*()).
//Execute the statement, usually in a loop with sqlite3_step().
//Process the results.
//You MUST call sqlite3_finalize(ppStmt) to delete the statement and release the memory. Failing to do this will cause a resource leak.

#[allow(dead_code)]
impl Connection {
    fn prepare(&self, sql_query: &str) -> Result<Statement, String> {
        let c_sql_query = CString::new(sql_query).expect("CString::new failed");
        let mut stmt = ptr::null_mut();
        let code = unsafe {
            ffi::sqlite3_prepare_v2(
                self.handle,
                c_sql_query.as_ptr(),
                -1,
                &mut stmt,
                ptr::null_mut(),
            )
        };
        if code == ffi::SQLITE_OK {
            Ok(Statement { stmt })
        } else {
            unsafe {
                // sqlite says we should not  drop c_error_msg
                let c_error_msg = ffi::sqlite3_errstr(code);
                let safe_error_msg = CStr::from_ptr(c_error_msg);
                let error_msg = safe_error_msg.to_string_lossy().into_owned();

                // ffi::sqlite3_close(db_handle);
                Err(error_msg)
            }
        }
    }
}
