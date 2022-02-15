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
use serde::{Deserialize, Serialize};

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

#[derive(Clone, Debug)]
/// Data required to create a gist in DB
/// creation date defaults to time at which creation method is called
pub struct CreateGist<'a> {
    /// owner of the gist
    pub owner: &'a str,
    /// description of the gist
    pub description: Option<&'a str>,
    /// public ID of the gist
    pub public_id: &'a str,
    /// gist visibility
    pub visibility: &'a GistVisibility,
}

/// Gist visibility
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GistVisibility {
    /// Everyone can see the gist, will be displayed on /explore and
    /// search engines might index it too
    Public,
    /// Everyone with the link can see it, won't be listed on /explore and
    /// search engines won't index them
    Unlisted,
    /// Only the owner can see gist
    Private,
}

impl GistVisibility {
    /// Convert [GistVisibility] to [str]
    pub const fn to_str(&self) -> &'static str {
        match self {
            GistVisibility::Private => "private",
            GistVisibility::Unlisted => "unlisted",
            GistVisibility::Public => "public",
        }
    }

    /// Convert [str] to [GistVisibility]
    pub fn from_str(s: &str) -> DBResult<Self> {
        const PRIVATE: &str = GistVisibility::Private.to_str();
        const PUBLIC: &str = GistVisibility::Public.to_str();
        const UNLISTED: &str = GistVisibility::Unlisted.to_str();
        let s = s.trim();
        match s {
            PRIVATE => Ok(Self::Private),
            PUBLIC => Ok(Self::Public),
            UNLISTED => Ok(Self::Unlisted),
            _ => Err(DBError::UnknownVisibilitySpecifier(s.to_owned())),
        }
    }
}

impl From<GistVisibility> for String {
    fn from(gp: GistVisibility) -> String {
        gp.to_str().into()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Represents a gist
pub struct Gist {
    /// owner of the gist
    pub owner: String,
    /// description of the gist
    pub description: Option<String>,
    /// public ID of the gist
    pub public_id: String,
    /// gist creation time
    pub created: i64,
    /// gist updated time
    pub updated: i64,
    /// gist visibility
    pub visibility: GistVisibility,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Represents a comment on a Gist
pub struct GistComment {
    /// Unique identifier, possible database assigned, auto-incremented ID
    pub id: i64,
    /// owner of the comment
    pub owner: String,
    /// public ID of the gist on which this comment was made
    pub gist_public_id: String,
    /// comment text
    pub comment: String,
    /// comment creation time
    pub created: i64,
}

#[derive(Clone, Debug)]
/// Data required to create a comment on a Gist
/// creation date defaults to time at which creation method is called
pub struct CreateGistComment<'a> {
    /// owner of the comment
    pub owner: &'a str,
    /// public ID of the gist on which this comment was made
    pub gist_public_id: &'a str,
    /// comment text
    pub comment: &'a str,
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
pub trait GistDatabase: std::marker::Send + std::marker::Sync + CloneGistDatabase {
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
    /// ping DB
    async fn ping(&self) -> bool;

    /// Check if a Gist with the given ID exists
    async fn gist_exists(&self, public_id: &str) -> DBResult<bool>;
    /// Create new gists
    async fn new_gist(&self, gist: &CreateGist) -> DBResult<()>;
    /// Retrieve gist from database
    async fn get_gist(&self, public_id: &str) -> DBResult<Gist>;

    /// Retrieve gists belonging to user
    async fn get_user_gists(&self, owner: &str) -> DBResult<Vec<Gist>>;

    /// Delete gist
    async fn delete_gist(&self, owner: &str, public_id: &str) -> DBResult<()>;

    /// Create new comment
    async fn new_comment(&self, comment: &CreateGistComment) -> DBResult<()>;
    /// Get comments on a gist
    async fn get_comments_on_gist(&self, public_id: &str) -> DBResult<Vec<GistComment>>;
    /// Get a specific comment using its database assigned ID
    async fn get_comment_by_id(&self, id: i64) -> DBResult<GistComment>;
    /// Delete comment
    async fn delete_comment(&self, owner: &str, id: i64) -> DBResult<()>;

    /// check if visibility mode exists
    async fn visibility_exists(&self, visibility: &GistVisibility) -> DBResult<bool>;
}

#[async_trait]
impl GistDatabase for Box<dyn GistDatabase> {
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

    /// ping DB
    async fn ping(&self) -> bool {
        (**self).ping().await
    }

    async fn gist_exists(&self, public_id: &str) -> DBResult<bool> {
        (**self).gist_exists(public_id).await
    }

    async fn new_gist(&self, gist: &CreateGist) -> DBResult<()> {
        (**self).new_gist(gist).await
    }

    async fn get_gist(&self, public_id: &str) -> DBResult<Gist> {
        (**self).get_gist(public_id).await
    }

    async fn get_user_gists(&self, owner: &str) -> DBResult<Vec<Gist>> {
        (**self).get_user_gists(owner).await
    }

    async fn delete_gist(&self, owner: &str, public_id: &str) -> DBResult<()> {
        (**self).delete_gist(owner, public_id).await
    }

    async fn new_comment(&self, comment: &CreateGistComment) -> DBResult<()> {
        (**self).new_comment(comment).await
    }

    async fn get_comments_on_gist(&self, public_id: &str) -> DBResult<Vec<GistComment>> {
        (**self).get_comments_on_gist(public_id).await
    }

    async fn get_comment_by_id(&self, id: i64) -> DBResult<GistComment> {
        (**self).get_comment_by_id(id).await
    }

    async fn delete_comment(&self, owner: &str, id: i64) -> DBResult<()> {
        (**self).delete_comment(owner, id).await
    }

    async fn visibility_exists(&self, visibility: &GistVisibility) -> DBResult<bool> {
        (**self).visibility_exists(visibility).await
    }
}

/// Trait to clone GistDatabase
pub trait CloneGistDatabase {
    /// clone DB
    fn clone_db<'a>(&self) -> Box<dyn GistDatabase>;
}

impl<T> CloneGistDatabase for T
where
    T: GistDatabase + Clone + 'static,
{
    fn clone_db(&self) -> Box<dyn GistDatabase> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn GistDatabase> {
    fn clone(&self) -> Self {
        (**self).clone_db()
    }
}
