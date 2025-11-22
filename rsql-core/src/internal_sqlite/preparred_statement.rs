use libsqlite3_sys::{
    SQLITE_BUSY, SQLITE_CONSTRAINT_CHECK, SQLITE_CONSTRAINT_FOREIGNKEY, SQLITE_CONSTRAINT_UNIQUE,
    SQLITE_DONE, SQLITE_OK, sqlite3, sqlite3_clear_bindings, sqlite3_reset, sqlite3_step,
    sqlite3_stmt,
};

use crate::{
    errors::{SqliteFailure, statement::StatementStepErrors},
    traits::to_sql::ToSql,
    utility::utils::get_sqlite_failiure,
};

pub struct PreparredStmt {
    pub stmt: *mut sqlite3_stmt,
    pub conn: *mut sqlite3,
}

impl Drop for PreparredStmt {
    fn drop(&mut self) {
        unsafe {
            sqlite3_reset(self.stmt);
            sqlite3_clear_bindings(self.stmt);
        }
    }
}

impl PreparredStmt {
    pub fn bind_parameter(&self, index: i32, value: impl ToSql) -> Result<(), SqliteFailure> {
        let code = unsafe { value.bind_to(self.stmt, index) };

        if code != SQLITE_OK {
            let (code, error_msg) = unsafe { get_sqlite_failiure(self.conn) };
            Err(SqliteFailure { code, error_msg })
        } else {
            Ok(())
        }
    }

    /// Strictly only used for write only operation (UPDATE, INSERT etc.)
    /// TODO: do we need to warn whether returns nothing? like during compile time check
    pub fn step(&mut self) -> Result<(), StatementStepErrors> {
        // TODO error handling?
        let code = unsafe { sqlite3_step(self.stmt) };

        if code == SQLITE_BUSY {
            return Err(StatementStepErrors::SqliteBusy);
        }

        let (code, error_msg) = unsafe { get_sqlite_failiure(self.conn) };

        if code == SQLITE_CONSTRAINT_FOREIGNKEY {
            Err(StatementStepErrors::ForeignKeyConstraint { code, error_msg })
        } else if code == SQLITE_CONSTRAINT_UNIQUE {
            Err(StatementStepErrors::UniqueConstraint { code, error_msg })
        } else if code == SQLITE_CONSTRAINT_CHECK {
            Err(StatementStepErrors::CheckConstraint { code, error_msg })
        } else if code == SQLITE_DONE {
            Ok(())
        } else {
            Err(StatementStepErrors::SqliteFailure { code, error_msg })
        }
    }

    // pub fn query<M: RowMapper>(self, mapper: M) -> Rows<'a, M> {
    //     Rows { stmt: self, mapper }
    // }
}
