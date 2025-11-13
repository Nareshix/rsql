use std::ffi::c_int;

use thiserror::Error;


/// Enum listing possible errors.

#[derive(Error, Debug)]
pub enum Error {
    /// Fail to open db
    //TODO error handling should be using direct String
    #[error("SQLite is unable to allocate memory to hold the database connection object")]
    ConnectionAllocationFailed,
    
    /// An error msg from an underlying SQLite call.
    //TODO error handling should be using direct String
    #[error("SQLite error {code}: {error_msg}")]
    SqliteFailure {
        code: c_int,
        error_msg: String,
    },

    #[error("Failed to convert sqlite data type to Rust data type")]
    SqliteToRustConversionFailiure
    // /// Error converting a C-style string from SQLite to a Rust `String`
    // /// because it was not valid UTF-8.
    // Utf8(Utf8Error),

    // /// Error converting a Rust `String` to a C-style string because it
    // /// contained an interior NUL (`\0`) byte.
    // Nul(NulError),


    // /// Error when a value from the database cannot be converted to the
    // /// requested Rust type.
    // FromSqlConversion {
    //     column_index: usize,
    //     source: Box<dyn error::Error + Send + Sync + 'static>,
    // },

    // /// Error when a Rust type cannot be converted into a value for the database.
    // /// This happens when binding parameters to a statement.
    // ToSqlConversion(Box<dyn error::Error + Send + Sync + 'static>),

    

    // /// Error when using named parameters and a provided name is not in the SQL.
    // InvalidParameterName(String),

    // /// Error when the number of bound parameters does not match the SQL query.
    // InvalidParameterCount { expected: usize, actual: usize },

    // /// Error when a column is requested by an index that is out of bounds.
    // InvalidColumnIndex(usize),

    // /// Error when a column is requested by a name that does not exist.
    // InvalidColumnName(String),

    // /// Error for a function like `query_row` that expects exactly one row
    // /// but got zero.
    // QueryReturnedNoRows,

    // /// Error for a function like `execute` (for inserts/updates) that was
    // // used with a `SELECT` statement that returned rows.
    // ExecuteReturnedResults,
}

// You would then implement `From` traits for `io::Error`, `Utf8Error`, `NulError`, etc.
// to make error handling ergonomic.



