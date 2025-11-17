use std::ffi::c_int;



pub mod connection;
pub mod row;
pub mod statement;

#[derive(thiserror::Error, Debug)]
#[error("SQLite error {code}: {error_msg}")]
pub struct SqliteFailure {
    pub code: c_int,
    pub error_msg: String,
}

