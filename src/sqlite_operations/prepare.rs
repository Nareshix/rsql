use libsqlite3_sys as ffi;

use std::{
    ffi::{ CString},
    ptr,
};

use crate::sqlite_operations::{connection::Connection, utils::detailed_error_msg};

#[allow(dead_code)]
pub struct Statement {
    stmt: *mut ffi::sqlite3_stmt,
}

//maybe dont add drop trait

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

        // If the input text contains no SQL (empty string or comment)
        //  stmt is set to NULL
        if code == ffi::SQLITE_OK && stmt.is_null(){
            let code = unsafe {
                 ffi::sqlite3_finalize( stmt)
            };

            if code == ffi::SQLITE_OK {
                
            }
        }

        else if code == ffi::SQLITE_OK {
            Ok(Statement { stmt })
        } else {
            unsafe {
                let error_msg = detailed_error_msg(self.handle);
                Err(error_msg)
            }
        }
    }
}
