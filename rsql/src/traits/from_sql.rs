use libsqlite3_sys::sqlite3_stmt;

pub trait FromSql {
    /// # Safety
    /// The caller must ensure that `stmt` points to a valid, stepped statement
    /// that is currently on a row (i.e., sqlite3_step has returned SQLITE_ROW).
    unsafe fn from_statement(stmt: *mut sqlite3_stmt) -> Self;
}


pub trait RowMapper {
    type Output;

    /// # Unsafe
    /// The caller must ensure the statement has a row ready to be read.
    #[allow(clippy::missing_safety_doc)]
    unsafe fn map_row(&self, stmt: *mut sqlite3_stmt) -> Self::Output;
}