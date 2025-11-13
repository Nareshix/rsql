use libsqlite3_sys::sqlite3_stmt;

pub trait RowMapper {
    type Output;

    /// # Unsafe
    /// The caller must ensure the statement has a row ready to be read.
    #[allow(clippy::missing_safety_doc)]
    unsafe fn map_row(&self, stmt: *mut sqlite3_stmt) -> Self::Output;
}