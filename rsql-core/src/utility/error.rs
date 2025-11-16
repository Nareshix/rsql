use std::ffi::c_int;

/// recommended to **log** this error rather than directly handling it
#[derive(thiserror::Error, Debug)]
#[error("SQLite error {code}: {error_msg}")]
pub struct SqliteFailure {
    pub code: c_int,
    pub error_msg: String,
}

// This errors
#[derive(thiserror::Error, Debug)]
pub enum SqliteOpenErrors {
    /// This error occurs when SQLite is unable to allocate memory to hold
    /// the database connection object. In other words, the device where this porgram is running does not
    /// have enough ram
    #[error("SQLite is unable to allocate memory to hold the database connection object")]
    ConnectionAllocationFailed,

    // in case of any other errors
    #[error("SQLite error {code}: {error_msg}")]
    SqliteFailure { code: c_int, error_msg: String },
}

#[derive(thiserror::Error, Debug)]
#[error("Failed to convert sqlite data type to Rust data type")]
pub struct SqliteToRustConversionFailiure;
