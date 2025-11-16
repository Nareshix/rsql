use std::ffi::c_int;

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
