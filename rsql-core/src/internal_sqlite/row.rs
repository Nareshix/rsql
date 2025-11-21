use libsqlite3_sys::{SQLITE_BUSY, SQLITE_DONE, SQLITE_ROW, sqlite3_step};

use crate::{
    errors::row::RowMapperError, internal_sqlite::statement::Statement,
    traits::row_mapper::RowMapper, utility::utils::get_sqlite_failiure,
};

#[allow(dead_code)]
pub struct Rows<'a, M: RowMapper> {
    pub stmt: Statement<'a>,
    pub mapper: M,
}



//TODO wht if rowmapper goes out of scope? wht owuld u do with the sqlite3_stmt
impl<'a, M: RowMapper> Iterator for Rows<'a, M> {
    // The Output refers to the original struct predefined by user (TODO, better explanation)
    type Item = Result<M::Output, RowMapperError>;

    fn next(&mut self) -> Option<Self::Item> {
        let result_code = unsafe { sqlite3_step(self.stmt.stmt) };

        if result_code == SQLITE_ROW {
            // Call the map_row method on our stored mapper instance.
            let item = unsafe { self.mapper.map_row(self.stmt.stmt) };
            Some(Ok(item))
        } else if result_code == SQLITE_BUSY {
            Some(Err(RowMapperError::SqliteBusy))
        } else if result_code == SQLITE_DONE {
            // when we exhaused the iterator, it has no use anymore. so we immediately reset it
            self.stmt.reset();
            None
        } else {
            let (code, error_msg) = unsafe { get_sqlite_failiure(self.stmt.conn.db) };
            Some(Err(RowMapperError::SqliteFailure { code, error_msg }))
        }
    }
}
