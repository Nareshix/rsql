use std::ffi::c_int;

use crate::errors::{connection::SqlitePrepareErrors, statement::StatementStepErrors};

pub mod connection;
pub mod row;
pub mod statement;

#[derive(thiserror::Error, Debug)]
#[error("SQLite error {code}: {error_msg}")]
pub struct SqliteFailure {
    pub code: c_int,
    pub error_msg: String,
}

#[derive(thiserror::Error, Debug)]
pub enum SqlWriteError {
    #[error("Failed to prepare statement: {0}")]
    Prepare(#[from] SqlitePrepareErrors),

    #[error("Failed to execute statement step: {0}")]
    Step(#[from] StatementStepErrors),
}

#[derive(thiserror::Error, Debug)]
pub enum SqlWriteBindingError {
    #[error("Failed to prepare statement: {0}")]
    Prepare(#[from] SqlitePrepareErrors),

    #[error("Failed to execute statement step: {0}")]
    Step(#[from] StatementStepErrors),

    #[error("Failed to Bind: {0}")]
    Bind(#[from] SqliteFailure),
}

#[derive(thiserror::Error, Debug)]
pub enum SqlReadError {
    #[error("Failed to prepare statement: {0}")]
    Prepare(#[from] SqlitePrepareErrors),
}

#[derive(thiserror::Error, Debug)]
pub enum SqlReadErrorBindings {
    #[error("Failed to prepare statement: {0}")]
    Prepare(#[from] SqlitePrepareErrors),

    #[error("Failed to Bind: {0}")]
    Bind(#[from] SqliteFailure),
}
