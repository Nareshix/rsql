use libsqlite3_sys::{SQLITE_ROW, sqlite3_step};

use crate::{internal_sqlite::statement::Statement, traits::row_mapper::RowMapper};



#[allow(dead_code)]
pub struct Rows<'a, M: RowMapper> {
    pub stmt: &'a Statement<'a>,
    pub mapper: M,
}

impl<'a, M: RowMapper> Iterator for Rows<'a, M> {
    // The Output refers to the original struct predefined by user (TODO, better explanation)
    type Item = M::Output;

    fn next(&mut self) -> Option<Self::Item> {
        let result_code = unsafe { sqlite3_step(self.stmt.stmt) };

        if result_code == SQLITE_ROW {
            // Call the map_row method on our stored mapper instance.
            let item = unsafe { self.mapper.map_row(self.stmt.stmt) };
            Some(item)
        } else {
            // SQLITE_DONE or an error occurred. TODO
            None
        }
    }
}
