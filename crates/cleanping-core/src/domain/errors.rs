//! Expected failures. Messages are user-facing and never contain secrets.

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CleanpingError {
    /// A user-supplied value violates a domain invariant.
    #[error("{0}")]
    Validation(String),
    /// A stored record referenced by id no longer exists.
    #[error("{0}")]
    NotFound(String),
    /// The request cannot run because no API key is available for it.
    #[error("{0}")]
    MissingCredential(String),
    /// The provider could not produce an edit (safe message only).
    #[error("{0}")]
    Rewrite(String),
    /// A local store (database or secret file) could not be read or written.
    #[error("{0}")]
    Storage(String),
}

pub type Result<T> = std::result::Result<T, CleanpingError>;
