#![feature(prelude_import)]
#[macro_use]
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use macros::execute;
use rsql::internal_sqlite::connection::Connection;
#[allow(unused)]
struct User;
#[automatically_derived]
#[allow(unused)]
impl ::core::marker::StructuralPartialEq for User {}
#[automatically_derived]
#[allow(unused)]
impl ::core::cmp::PartialEq for User {
    #[inline]
    fn eq(&self, other: &User) -> bool {
        true
    }
}
#[automatically_derived]
#[allow(unused)]
impl ::core::fmt::Debug for User {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::write_str(f, "User")
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open("hi.db")?;
    let name = "mom";
    let email = "mom@example.com";
    let stmt = conn.prepare("INSERT INTO users (username, email) VALUES (?, ?)")?;
    stmt.bind_parameter(1i32, name)?;
    stmt.bind_parameter(2i32, email)?;
    stmt.step();
    Ok(())
}
