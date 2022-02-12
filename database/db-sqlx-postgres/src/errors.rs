//! Error-handling utilities
use std::borrow::Cow;

use db_core::dev::*;
use sqlx::Error;

/// map postgres errors to [DBError](DBError) types
pub fn map_register_err(e: Error) -> DBError {
    if let Error::Database(err) = e {
        if err.code() == Some(Cow::from("23505")) {
            let msg = err.message();
            if msg.contains("admin_users_username_key") {
                DBError::DuplicateUsername
            } else if msg.contains("admin_users_email_key") {
                DBError::DuplicateEmail
            } else if msg.contains("admin_users_secret_key") {
                DBError::DuplicateSecret
            } else {
                DBError::DBError(Box::new(Error::Database(err)))
            }
        } else {
            DBError::DBError(Box::new(Error::Database(err)))
        }
    } else {
        DBError::DBError(Box::new(e))
    }
}
