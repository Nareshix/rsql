use libsqlite3_sys::{
    SQLITE_BUSY, SQLITE_CONSTRAINT_CHECK, SQLITE_CONSTRAINT_FOREIGNKEY, SQLITE_CONSTRAINT_UNIQUE,
    SQLITE_DONE, SQLITE_OK, sqlite3_clear_bindings, sqlite3_finalize, sqlite3_reset, sqlite3_step,
    sqlite3_stmt,
};

use crate::{
    errors::{SqliteFailure, statement::StatementStepErrors},
    internal_sqlite::row::Rows,
    traits::{row_mapper::RowMapper, to_sql::ToSql},
    utility::utils::get_sqlite_failiure,
};

use crate::internal_sqlite::connection::Connection;

#[allow(dead_code)]
// #[derive(Debug)]
pub struct Statement<'a> {
    pub(crate) conn: &'a Connection,
    pub(crate) stmt: *mut sqlite3_stmt,
    pub(crate) key: Option<String>,
}

impl Statement<'_> {
    fn reset(&self) {
        // If key exists, the stmt exists in the cache
        if let Some(ref key) = self.key {
            let mut cache = self.conn.cache.borrow_mut();

            if let Some(cached_stmt) = cache.get_mut(key) {
                cached_stmt.in_use = false;
                unsafe {
                    sqlite3_reset(self.stmt);
                    sqlite3_clear_bindings(self.stmt);
                }
            }
        } else {
            // this stmt doesnt live in the cache, so we have to manually destroy it
            unsafe {
                sqlite3_finalize(self.stmt);
            }
        }
    }
}

impl Drop for Statement<'_> {
    fn drop(&mut self) {
        self.reset();
    }
}

impl Statement<'_> {
    //TODO
    //If any of the sqlite3_bind_*() routines are called with a NULL pointer for the
    // prepared statement or with a prepared statement for which sqlite3_step()
    // has been called more recently than sqlite3_reset(), then the call will return SQLITE_MISUSE.
    // If any sqlite3_bind_() routine is passed a prepared statement that has been finalized,
    // the result is undefined and probably harmful.

    ///note index start from 1 and not 0
    /// TODO consider &impl ToSql to prevent moving?
    #[allow(unused)]
    pub fn bind_parameter(&self, index: i32, value: impl ToSql) -> Result<(), SqliteFailure> {
        let code = unsafe { value.bind_to(self.stmt, index) };

        if code != SQLITE_OK {
            let (code, error_msg) = unsafe { get_sqlite_failiure(self.conn.db) };
            Err(SqliteFailure { code, error_msg })
        } else {
            Ok(())
        }
    }

    /// Strictly only used for write only operation (UPDATE, INSERT etc.)
    /// TODO: do we need to warn whether returns nothing? like during compile time check
    pub fn step(&self) -> Result<(), StatementStepErrors> {
        // TODO error handling?
        let code = unsafe { sqlite3_step(self.stmt) };

        if code == SQLITE_BUSY {
            return Err(StatementStepErrors::SqliteBusy);
        }

        let (code, error_msg) = unsafe { get_sqlite_failiure(self.conn.db) };

        if code == SQLITE_CONSTRAINT_FOREIGNKEY {
            Err(StatementStepErrors::ForeignKeyConstraint { code, error_msg })
        } else if code == SQLITE_CONSTRAINT_UNIQUE {
            Err(StatementStepErrors::UniqueConstraint { code, error_msg })
        } else if code == SQLITE_CONSTRAINT_CHECK {
            Err(StatementStepErrors::CheckConstraint { code, error_msg })
        } else if code == SQLITE_DONE {
            // Since step is one time use, its safe to immediately reset the stmt
            self.reset();
            Ok(())
        } else {
            Err(StatementStepErrors::SqliteFailure { code, error_msg })
        }
    }

    pub fn query<'a, M: RowMapper>(&'a self, mapper: M) -> Rows<'a, M> {
        Rows { stmt: self, mapper }
    }
}
