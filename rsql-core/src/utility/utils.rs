use libsqlite3_sys::{
    self as ffi, SQLITE_OK, SQLITE_OPEN_CREATE, SQLITE_OPEN_MEMORY, SQLITE_OPEN_READONLY,
    SQLITE_OPEN_READWRITE, SQLITE_ROW, sqlite3, sqlite3_close, sqlite3_column_text,
    sqlite3_errcode, sqlite3_exec, sqlite3_finalize, sqlite3_free, sqlite3_open_v2,
    sqlite3_prepare_v2, sqlite3_step, sqlite3_stmt,
};
use std::{
    ffi::{CStr, CString, c_char, c_void},
    fs,
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
    let path = Path::new(db_path);

    if !path.exists() {
        return Err(SqliteOpenErrors::SqliteFailure {
            code: 0,
            error_msg: format!("File not found at path: '{}'", db_path),
        });
    }

    let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let is_sql_script = extension.eq_ignore_ascii_case("sql");

    let mut db = ptr::null_mut();

    unsafe {
        let rc = if is_sql_script {
            let memory_path = CString::new(":memory:").unwrap();
            let flags = SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE | SQLITE_OPEN_MEMORY;
            sqlite3_open_v2(memory_path.as_ptr(), &mut db, flags, ptr::null())
        } else {
            let c_path = CString::new(db_path).unwrap();
            let flags = SQLITE_OPEN_READONLY;
            sqlite3_open_v2(c_path.as_ptr(), &mut db, flags, ptr::null())
        };

        if rc != SQLITE_OK {
            let (code, error_msg) = get_sqlite_failiure(db);
            sqlite3_close(db);
            return Err(SqliteOpenErrors::SqliteFailure { code, error_msg });
        }

        if is_sql_script {
            let file_content =
                fs::read_to_string(path).map_err(|e| SqliteOpenErrors::SqliteFailure {
                    code: 0,
                    error_msg: format!("Failed to read .sql file: {}", e),
                })?;

            let c_sql =
                CString::new(file_content).map_err(|_| SqliteOpenErrors::SqliteFailure {
                    code: 0,
                    error_msg: "SQL file contains illegal null bytes".to_string(),
                })?;

            let mut err_msg: *mut c_char = ptr::null_mut();
            let exec_rc = sqlite3_exec(db, c_sql.as_ptr(), None, ptr::null_mut(), &mut err_msg);

            if exec_rc != SQLITE_OK {
                let msg = if !err_msg.is_null() {
                    let m = CStr::from_ptr(err_msg).to_string_lossy().into_owned();
                    sqlite3_free(err_msg as *mut c_void); // Fix memory leak
                    m
                } else {
                    "Unknown error".to_string()
                };
                sqlite3_close(db);
                return Err(SqliteOpenErrors::SqliteFailure {
                    code: exec_rc,
                    error_msg: format!("Error in .sql script: {}", msg),
                });
            }
        }

        let sql = b"SELECT name, sql FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name\0";
        let mut stmt: *mut sqlite3_stmt = ptr::null_mut();

        let prepare_rc = sqlite3_prepare_v2(
            db,
            sql.as_ptr() as *const c_char,
            -1,
            &mut stmt,
            ptr::null_mut(),
        );

        if prepare_rc != SQLITE_OK {
            let (code, error_msg) = get_sqlite_failiure(db);
            sqlite3_close(db);
            return Err(SqliteOpenErrors::SqliteFailure { code, error_msg });
        }

        let mut results = Vec::new();
        while sqlite3_step(stmt) == SQLITE_ROW {
            let sql_ptr = sqlite3_column_text(stmt, 1);
            let sql_txt = if sql_ptr.is_null() {
                String::new()
            } else {
                CStr::from_ptr(sql_ptr as *const c_char)
                    .to_string_lossy()
                    .into_owned()
            };
            results.push(sql_txt);
        }

        sqlite3_finalize(stmt);
        sqlite3_close(db);

        Ok(results)
    }
}

pub fn validate_sql_syntax_with_sqlite(db_path: &str, sql: &str) -> Result<(), String> {
    let path = Path::new(db_path);

    if !path.exists() {
        return Err(format!("File not found at path: '{}'", db_path));
    }

    let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let is_sql_script = extension.eq_ignore_ascii_case("sql");

    unsafe {
        let mut db: *mut sqlite3 = ptr::null_mut();

        let rc = if is_sql_script {
            let memory_path = CString::new(":memory:").unwrap();
            let flags = SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE | SQLITE_OPEN_MEMORY;
            sqlite3_open_v2(memory_path.as_ptr(), &mut db, flags, ptr::null())
        } else {
            let c_path = CString::new(db_path)
                .map_err(|_| "Invalid DB path (contains null byte)".to_string())?;
            let flags = SQLITE_OPEN_READONLY;
            sqlite3_open_v2(c_path.as_ptr(), &mut db, flags, ptr::null())
        };

        if rc != SQLITE_OK {
            let (code, msg) = get_sqlite_failiure(db);
            sqlite3_close(db);
            return Err(format!("Failed to open DB. Code: {}. Error: {}", code, msg));
        }

        if is_sql_script {
            let file_content = fs::read_to_string(path).map_err(|e| {
                sqlite3_close(db);
                format!("Failed to read .sql file: {}", e)
            })?;

            let c_sql = CString::new(file_content).map_err(|_| {
                sqlite3_close(db);
                "SQL file contains illegal null bytes".to_string()
            })?;

            let mut err_msg: *mut c_char = ptr::null_mut();
            let exec_rc = sqlite3_exec(db, c_sql.as_ptr(), None, ptr::null_mut(), &mut err_msg);

            if exec_rc != SQLITE_OK {
                let msg = if !err_msg.is_null() {
                    let m = CStr::from_ptr(err_msg).to_string_lossy().into_owned();
                    sqlite3_free(err_msg as *mut c_void);
                    m
                } else {
                    "Unknown error".to_string()
                };
                sqlite3_close(db);
                return Err(format!("Error setting up schema from .sql: {}", msg));
            }
        }

        let c_sql =
            CString::new(sql).map_err(|_| "Invalid SQL string (contains null byte)".to_string())?;

        let mut stmt = ptr::null_mut();
        let mut tail = ptr::null();

        let prepare_rc = sqlite3_prepare_v2(db, c_sql.as_ptr(), -1, &mut stmt, &mut tail);

        let result = if prepare_rc == SQLITE_OK {
            Ok(())
        } else {
            let (code, msg) = get_sqlite_failiure(db);
            Err(format!("Validation Error (Code {}): {}", code, msg))
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

            libsqlite3_sys::sqlite3_open(c_name.as_ptr(), &mut db);

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
