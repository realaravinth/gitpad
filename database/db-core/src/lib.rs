#![warn(missing_docs)]
//! # `gists` database operations
//!
//! Traits and datastructures used in gists to interact with database.
//!
//! To use an unsupported database with gists, traits present within this crate should be
//! implemented.
//!
//!
//! ## Organisation
//!
//! Database functionallity is divided accross various modules:
//!
//! - [errors](crate::auth): error data structures used in this crate
//! - [ops](crate::ops): meta operations like connection pool creation, migrations and getting
//! connection from pool
pub mod errors;
pub mod ops;
#[cfg(feature = "test")]
pub mod tests;

pub use ops::GetConnection;

pub mod prelude {
    //! useful imports for users working with a supported database

    pub use super::errors::*;
    pub use super::ops::*;
    pub use super::*;
}

pub mod dev {
    //! useful imports for supporting a new database
    pub use super::prelude::*;
    pub use async_trait::async_trait;
}

/// data structure describing credentials of a user
#[derive(Clone, Debug)]
pub struct Creds {
    /// username
    pub username: String,
    /// password
    pub password: String,
}

/// data structure containing only a password field
#[derive(Clone, Debug)]
pub struct Password {
    /// password
    pub password: String,
}

/// payload to register a user with username _and_ email
pub struct EmailRegisterPayload<'a> {
    /// username of new user
    pub username: &'a str,
    /// password of new user
    pub password: &'a str,
    /// password of new user
    pub email: &'a str,
    /// a randomly generated secret associated with an account
    pub secret: &'a str,
}

/// payload to register a user with only username
pub struct UsernameRegisterPayload<'a> {
    /// username provided during registration
    pub username: &'a str,
    /// password of new user
    pub password: &'a str,
    /// a randomly generated secret associated with an account
    pub secret: &'a str,
}

/// payload to update email in the database
#[derive(Clone, Debug)]
pub struct UpdateEmailPayload<'a> {
    /// name of the user who's email is to be updated
    pub username: &'a str,
    /// new email
    pub email: &'a str,
}

/// payload to update a username in database
pub struct UpdateUsernamePayload<'a> {
    /// old usename
    pub old_username: &'a str,
    /// new username
    pub new_username: &'a str,
}

use dev::*;
/// foo
#[async_trait]
pub trait GistDatabase: std::marker::Send + std::marker::Sync {
    /// Update email of specified user in database
    async fn update_email(&self, payload: &UpdateEmailPayload) -> DBResult<()>;
    /// Update password of specified user in database
    async fn update_password(&self, payload: &Creds) -> DBResult<()>;
    /// check if an email exists in the database
    async fn email_exists(&self, email: &str) -> DBResult<bool>;
    /// delete account from database
    async fn delete_account(&self, username: &str) -> DBResult<()>;
    /// check if a username exists in the database
    async fn username_exists(&self, username: &str) -> DBResult<bool>;
    /// update username in database
    async fn update_username(&self, payload: &UpdateUsernamePayload) -> DBResult<()>;
    /// update secret in database
    async fn update_secret(&self, username: &str, secret: &str) -> DBResult<()>;
    /// update secret in database
    async fn get_secret(&self, username: &str) -> DBResult<String>;
    /// login with email as user-identifier
    async fn email_login(&self, email: &str) -> DBResult<Creds>;
    /// login with username as user-identifier
    async fn username_login(&self, username: &str) -> DBResult<Password>;
    /// username _and_ email is available during registration
    async fn email_register(&self, payload: &EmailRegisterPayload) -> DBResult<()>;
    /// register with username
    async fn username_register(&self, payload: &UsernameRegisterPayload) -> DBResult<()>;
}

#[async_trait]
impl<T: GistDatabase + ?Sized> GistDatabase for Box<T> {
    async fn update_email(&self, payload: &UpdateEmailPayload) -> DBResult<()> {
        (**self).update_email(payload).await
    }
    /// Update password of specified user in database
    async fn update_password(&self, payload: &Creds) -> DBResult<()> {
        (**self).update_password(payload).await
    }
    /// check if an email exists in the database
    async fn email_exists(&self, email: &str) -> DBResult<bool> {
        (**self).email_exists(email).await
    }
    /// delete account from database
    async fn delete_account(&self, username: &str) -> DBResult<()> {
        (**self).delete_account(username).await
    }
    /// check if a username exists in the database
    async fn username_exists(&self, username: &str) -> DBResult<bool> {
        (**self).username_exists(username).await
    }
    /// update username in database
    async fn update_username(&self, payload: &UpdateUsernamePayload) -> DBResult<()> {
        (**self).update_username(payload).await
    }
    /// update secret in database
    async fn update_secret(&self, username: &str, secret: &str) -> DBResult<()> {
        (**self).update_secret(username, secret).await
    }
    /// update secret in database
    async fn get_secret(&self, username: &str) -> DBResult<String> {
        (**self).get_secret(username).await
    }
    /// login with email as user-identifier
    async fn email_login(&self, email: &str) -> DBResult<Creds> {
        (**self).email_login(email).await
    }
    /// login with username as user-identifier
    async fn username_login(&self, username: &str) -> DBResult<Password> {
        (**self).username_login(username).await
    }
    /// username _and_ email is available during registration
    async fn email_register(&self, payload: &EmailRegisterPayload) -> DBResult<()> {
        (**self).email_register(payload).await
    }
    /// register with username
    async fn username_register(&self, payload: &UsernameRegisterPayload) -> DBResult<()> {
        (**self).username_register(payload).await
    }
}
