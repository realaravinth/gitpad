use std::borrow::Cow;

use db_core::dev::*;
use sqlx::Error;

pub fn map_register_err(e: Error) -> DBError {
    if let Error::Database(err) = e {
        if err.code() == Some(Cow::from("2067")) {
            let msg = err.message();
            println!("{}", msg);
            if msg.contains("gists_users.username") {
                DBError::DuplicateUsername
            } else if msg.contains("gists_users.email") {
                DBError::DuplicateEmail
            } else if msg.contains("gists_users.secret") {
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
