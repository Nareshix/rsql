use libsqlite3_sys::{
    self as ffi, SQLITE_OK, SQLITE_OPEN_CREATE, SQLITE_OPEN_READONLY, SQLITE_OPEN_READWRITE,
    SQLITE_ROW, sqlite3, sqlite3_busy_timeout, sqlite3_close, sqlite3_column_text, sqlite3_errcode,
    sqlite3_finalize, sqlite3_open, sqlite3_open_v2, sqlite3_prepare_v2, sqlite3_step,
    sqlite3_stmt,
};
use std::{
    ffi::{CStr, CString, c_char},
    path::Path,
    ptr,
};

use crate::errors::connection::{SqliteOpenErrors, SqlitePrepareErrors};

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

///# Safety
/// db must be a valid pointer
pub unsafe fn prepare_stmt(
    db: *mut sqlite3,
    stmt: &mut *mut sqlite3_stmt,
    sql: &str,
) -> Result<(), SqlitePrepareErrors> {
    let c_sql_query = CString::new(sql).unwrap(); //TODO
    let code =
        unsafe { ffi::sqlite3_prepare_v2(db, c_sql_query.as_ptr(), -1, stmt, ptr::null_mut()) };

    // TODO. In your macro, MUST make sure that the sql is not empty, no pure whitepaces and is not purely a comment
    if code != SQLITE_OK {
        let (code, error_msg) = unsafe { get_sqlite_failiure(db) };
        return Err(SqlitePrepareErrors::SqliteFailure { code, error_msg });
    }
    Ok(())
}

pub fn get_db_schema(db_path: &str) -> Result<Vec<String>, SqliteOpenErrors> {
    // TODO, there can be abetter more robust way of handling paths. maybe a .env file?, current_exe() or current_dir()?
    if !Path::new(db_path).exists() {
        return Err(SqliteOpenErrors::SqliteFailure {
            code: 0,
            error_msg: format!("File not found at path: '{}'", db_path),
        });
    }

    let c_path = CString::new(db_path).unwrap();
    let mut db = ptr::null_mut();
    let mut results = Vec::new();

    unsafe {
        let flags = SQLITE_OPEN_READONLY;
        let rc = sqlite3_open_v2(c_path.as_ptr(), &mut db, flags, ptr::null());

        if rc != SQLITE_OK {
            let (code, error_msg) = get_sqlite_failiure(db);
            sqlite3_close(db);
            return Err(SqliteOpenErrors::SqliteFailure { code, error_msg });
        }

        // 2. ADD TIMEOUT to stop "Database Locked" errors with VS Code
        // sqlite3_busy_timeout(db, 2000); // Wait 2 seconds

        let sql = b"SELECT name, sql FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name\0";
        let mut stmt: *mut sqlite3_stmt = ptr::null_mut();

        let prepare_rc = sqlite3_prepare_v2(
            db,
            sql.as_ptr() as *const i8,
            -1,
            &mut stmt,
            ptr::null_mut(),
        );

        if prepare_rc != SQLITE_OK {
            let (code, error_msg) = get_sqlite_failiure(db);
            sqlite3_close(db);
            return Err(SqliteOpenErrors::SqliteFailure { code, error_msg });
        }

        // Step through rows
        while sqlite3_step(stmt) == SQLITE_ROW {
            let sql_ptr = sqlite3_column_text(stmt, 1);
            let sql_txt = if sql_ptr.is_null() {
                String::new()
            } else {
                CStr::from_ptr(sql_ptr as *const i8)
                    .to_string_lossy()
                    .into_owned()
            };
            results.push(sql_txt);
        }

        sqlite3_finalize(stmt);
        sqlite3_close(db);
    }

    Ok(results)
}
pub fn validate_sql_syntax_with_sqlite(db_path: &str, sql: &str) -> Result<(), String> {
    unsafe {
        let c_db_path = CString::new(db_path)
            .map_err(|_| "Invalid DB path (contains null byte)".to_string())?;
        let c_sql =
            CString::new(sql).map_err(|_| "Invalid SQL string (contains null byte)".to_string())?;

        let mut db: *mut sqlite3 = ptr::null_mut();

        let open_flags = SQLITE_OPEN_READONLY;
        let open_rc = sqlite3_open_v2(c_db_path.as_ptr(), &mut db, open_flags, ptr::null());

        if open_rc != SQLITE_OK {
            let err_msg = get_sqlite_failiure(db);
            sqlite3_close(db);
            return Err(format!("Error: {}. {}", err_msg.0, err_msg.1));
        }

        // Wait up to 2000ms (2s) if the file is locked by VS Code
        // sqlite3_busy_timeout(db, 2000);

        let mut stmt = ptr::null_mut();
        let mut tail = ptr::null();

        let prepare_rc = sqlite3_prepare_v2(db, c_sql.as_ptr(), -1, &mut stmt, &mut tail);

        let result = if prepare_rc == SQLITE_OK {
            Ok(())
        } else {
            let err_msg = get_sqlite_failiure(db);
            Err(format!(
                "Error: {} Failed to prepare SQL: {}", // Changed message slightly
                err_msg.0, err_msg.1
            ))
        };

        if !stmt.is_null() {
            sqlite3_finalize(stmt);
        }

        sqlite3_close(db);
        result
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_schema_raw() {
        unsafe {
            let db_name = "test_schema.db";
            let c_name = CString::new(db_name).unwrap();
            let mut db: *mut sqlite3 = ptr::null_mut();

            sqlite3_open(c_name.as_ptr(), &mut db);

            let create_sql = b"CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)\0";
            let mut stmt: *mut sqlite3_stmt = ptr::null_mut();
            sqlite3_prepare_v2(
                db,
                create_sql.as_ptr() as *const i8,
                -1,
                &mut stmt,
                ptr::null_mut(),
            );
            sqlite3_step(stmt);
            sqlite3_finalize(stmt);
            sqlite3_close(db);

            let schema = get_db_schema(db_name).unwrap();

            println!("{schema:?}");
            assert_eq!(schema.len(), 1);
            assert!(schema[0].contains("CREATE TABLE users")); // SQL matches

            // Cleanup file
            std::fs::remove_file(db_name).unwrap_or(());
        }
    }
}
