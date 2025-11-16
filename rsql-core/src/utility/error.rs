use std::ffi::c_int;

/// This error occurs when SQLite is unable to allocate memory to hold
/// the database connection object. In other words, the device where this porgram is running does not
/// have enough ram
#[derive(thiserror::Error, Debug)]
#[error("SQLite is unable to allocate memory to hold the database connection object")]
pub struct ConnectionAllocationFailed;

/// recommended to **log** this error rather than directly handling it
#[derive(thiserror::Error, Debug)]
#[error("SQLite error {code}: {error_msg}")]
pub struct SqliteFailure {
    pub code: c_int,
    pub error_msg: String,
}

#[derive(thiserror::Error, Debug)]
pub enum SqliteOpenErrors {
    #[error("SQLite is unable to allocate memory to hold the database connection object")]
    ConnectionAllocationFailed,

    #[error("SQLite error {code}: {error_msg}")]
    SqliteFailure { code: c_int, error_msg: String },
}

#[derive(thiserror::Error, Debug)]
#[error("Failed to convert sqlite data type to Rust data type")]
pub struct SqliteToRustConversionFailiure;
