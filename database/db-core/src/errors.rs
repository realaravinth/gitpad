//! represents all the ways a trait can fail using this crate
use std::error::Error as StdError;

//use derive_more::{error, Error as DeriveError};
use thiserror::Error;

/// Error data structure grouping various error subtypes
#[derive(Debug, Error)]
pub enum DBError {
    /// username is already taken
    #[error("Username not available")]
    DuplicateUsername,

    /// user secret is already taken
    #[error("User secret not available")]
    DuplicateSecret,

    /// email is already taken
    #[error("Email not available")]
    DuplicateEmail,

    /// Gist public ID taken
    #[error("Gist ID not available")]
    GistIDTaken,

    /// Account with specified characteristics not found
    #[error("Account with specified characteristics not found")]
    AccountNotFound,

    /// errors that are specific to a database implementation
    #[error("{0}")]
    DBError(#[source] BoxDynError),

    /// email is already taken
    #[error("Unknown privacy specifier {}", _0)]
    UnknownPrivacySpecifier(String),

    /// Gist with specified characteristics not found
    #[error("Gist with specified characteristics not found")]
    GistNotFound,

    /// Comment with specified characteristics not found
    #[error("Comment with specified characteristics not found")]
    CommentNotFound,
}

/// Convenience type alias for grouping driver-specific errors
pub type BoxDynError = Box<dyn StdError + 'static + Send + Sync>;

/// Generic result data structure
pub type DBResult<V> = std::result::Result<V, DBError>;
