#![feature(prelude_import)]
#[macro_use]
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use macros::{SqlMapping, query};
use rsql::internal_sqlite::connection::Connection;
struct Person {
    id: i32,
    username: String,
    email: String,
}
#[automatically_derived]
impl ::core::fmt::Debug for Person {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field3_finish(
            f,
            "Person",
            "id",
            &self.id,
            "username",
            &self.username,
            "email",
            &&self.email,
        )
    }
}
use libsqlite3_sys::sqlite3_stmt;
use rsql::traits::row_mapper::RowMapper;
use rsql::traits::from_sql::FromSql;
struct PersonMapper;
impl RowMapper for PersonMapper {
    type Output = Person;
    unsafe fn map_row(&self, stmt: *mut sqlite3_stmt) -> Self::Output {
        let id = unsafe { i32::from_col(stmt, 0i32) };
        let username = unsafe { String::from_col(stmt, 1i32) };
        let email = unsafe { String::from_col(stmt, 2i32) };
        Self::Output {
            id,
            username,
            email,
        }
    }
}
#[allow(non_upper_case_globals)]
const Person: PersonMapper = PersonMapper;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open("hi.db").unwrap();
    let result = {
        let stmt = conn.prepare("SELECT * FROM users")?;
        stmt.query(Person)
    };
    Ok(())
}
