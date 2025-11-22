use libsqlite3_sys::sqlite3_stmt;

pub struct LazyStatement {
    stmt: Option<sqlite3_stmt>
}

// TODO impl Drop