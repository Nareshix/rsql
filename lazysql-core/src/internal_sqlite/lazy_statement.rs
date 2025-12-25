use libsqlite3_sys::{sqlite3_finalize, sqlite3_stmt};

#[allow(unused)]
pub struct LazyStmt {
    pub sql_query: &'static str,
    pub stmt: *mut sqlite3_stmt,
}

unsafe impl Send for LazyStmt {}
unsafe impl Sync for LazyStmt {}

impl Drop for LazyStmt {
    fn drop(&mut self) {
        // If the statement was initialized, we must finalize it to prevent memory leaks.
        if !self.stmt.is_null() {
            unsafe {
                sqlite3_finalize(self.stmt);
            }
        }
    }
}
