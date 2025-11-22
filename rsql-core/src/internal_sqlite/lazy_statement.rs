use libsqlite3_sys::sqlite3_stmt;

#[allow(unused)]
pub struct LazyStmt {
    pub sql_query: &'static str,
    pub stmt: *mut sqlite3_stmt,
}

// TODO impl Drop