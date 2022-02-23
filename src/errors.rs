/*
 * Copyright (C) 2022  Aravinth Manivannan <realaravinth@batsense.net>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
//! represents all the ways a trait can fail using this crate
use std::convert::From;
use std::io::Error as FSErrorInner;

use actix_web::{
    error::ResponseError,
    http::{header, StatusCode},
    HttpResponse, HttpResponseBuilder,
};
use argon2_creds::errors::CredsError;
use db_core::errors::DBError;
use derive_more::{Display, Error};
use serde::{Deserialize, Serialize};
use url::ParseError;
use validator::ValidationErrors;

#[derive(Debug, Display, Error)]
pub struct FSError(#[display(fmt = "File System Error {}", _0)] pub FSErrorInner);

#[cfg(not(tarpaulin_include))]
impl PartialEq for FSError {
    fn eq(&self, other: &Self) -> bool {
        self.0.kind() == other.0.kind()
    }
}

#[cfg(not(tarpaulin_include))]
impl From<FSErrorInner> for ServiceError {
    fn from(e: FSErrorInner) -> Self {
        Self::FSError(FSError(e))
    }
}

#[derive(Debug, PartialEq, Display, Error)]
#[cfg(not(tarpaulin_include))]
/// Error data structure grouping various error subtypes
pub enum ServiceError {
    /// All non-specific errors are grouped under this category
    #[display(fmt = "internal server error")]
    InternalServerError,

    #[display(
        fmt = "This server is is closed for registration. Contact admin if this is unexpecter"
    )]
    /// registration failure, server is is closed for registration
    ClosedForRegistration,

    #[display(fmt = "The value you entered for email is not an email")] //405j
    /// The value you entered for email is not an email"
    NotAnEmail,
    #[display(fmt = "The value you entered for URL is not a URL")] //405j
    /// The value you entered for url is not url"
    NotAUrl,
    #[display(fmt = "The value you entered for ID is not a valid ID")] //405j
    /// The value you entered for ID is not a valid ID
    NotAnId,
    #[display(fmt = "URL too long, maximum length can't be greater then 2048 characters")] //405
    /// URL too long, maximum length can't be greater then 2048 characters
    URLTooLong,

    #[display(fmt = "Wrong password")]
    /// wrong password
    WrongPassword,
    #[display(fmt = "Account not found")]
    /// account not found
    AccountNotFound,

    #[display(fmt = "Gist not found")]
    /// gist not found
    GistNotFound,

    #[display(fmt = "comment not found")]
    /// comment not found
    CommentNotFound,

    /// when the value passed contains profainity
    #[display(fmt = "Can't allow profanity in usernames")]
    ProfainityError,
    /// when the value passed contains blacklisted words
    /// see [blacklist](https://github.com/shuttlecraft/The-Big-Username-Blacklist)
    #[display(fmt = "Username contains blacklisted words")]
    BlacklistError,
    /// when the value passed contains characters not present
    /// in [UsernameCaseMapped](https://tools.ietf.org/html/rfc8265#page-7)
    /// profile
    #[display(fmt = "username_case_mapped violation")]
    UsernameCaseMappedError,

    #[display(fmt = "Passsword too short")]
    /// password too short
    PasswordTooShort,
    #[display(fmt = "password too long")]
    /// password too long
    PasswordTooLong,
    #[display(fmt = "Passwords don't match")]
    /// passwords don't match
    PasswordsDontMatch,

    /// when the a username is already taken
    #[display(fmt = "Username not available")]
    UsernameTaken,

    /// email is already taken
    #[display(fmt = "Email not available")]
    EmailTaken,

    #[display(fmt = "File System Error {}", _0)]
    FSError(FSError),

    #[display(fmt = "Comment is empty")]
    EmptyComment,

    #[display(fmt = "Unauthorized {}", _0)]
    UnauthorizedOperation(#[error(not(source))] String),

    #[display(fmt = "Bad request: {}", _0)]
    BadRequest(#[error(not(source))] String),

    #[display(fmt = "Gist is empty, at least one file is required to create gist")]
    GistEmpty,
}

impl From<CredsError> for ServiceError {
    #[cfg(not(tarpaulin_include))]
    fn from(e: CredsError) -> ServiceError {
        match e {
            CredsError::UsernameCaseMappedError => ServiceError::UsernameCaseMappedError,
            CredsError::ProfainityError => ServiceError::ProfainityError,
            CredsError::BlacklistError => ServiceError::BlacklistError,
            CredsError::NotAnEmail => ServiceError::NotAnEmail,
            CredsError::Argon2Error(_) => ServiceError::InternalServerError,
            CredsError::PasswordTooLong => ServiceError::PasswordTooLong,
            CredsError::PasswordTooShort => ServiceError::PasswordTooShort,
        }
    }
}

impl From<ValidationErrors> for ServiceError {
    #[cfg(not(tarpaulin_include))]
    fn from(_: ValidationErrors) -> ServiceError {
        ServiceError::NotAnEmail
    }
}

impl From<ParseError> for ServiceError {
    #[cfg(not(tarpaulin_include))]
    fn from(_: ParseError) -> ServiceError {
        ServiceError::NotAUrl
    }
}

impl From<DBError> for ServiceError {
    #[cfg(not(tarpaulin_include))]
    fn from(e: DBError) -> Self {
        log::error!("{:?}", e);
        println!("{:?}", e);
        match e {
            DBError::DBError(_) => ServiceError::InternalServerError,
            DBError::DuplicateEmail => ServiceError::EmailTaken,
            DBError::DuplicateUsername => ServiceError::UsernameTaken,
            DBError::AccountNotFound => ServiceError::AccountNotFound,
            DBError::DuplicateSecret => ServiceError::InternalServerError,
            DBError::GistNotFound => ServiceError::GistNotFound,
            DBError::CommentNotFound => ServiceError::CommentNotFound,
            DBError::GistIDTaken => ServiceError::InternalServerError,
            DBError::UnknownVisibilitySpecifier(_) => ServiceError::InternalServerError,
        }
    }
}

/// Generic result data structure
#[cfg(not(tarpaulin_include))]
pub type ServiceResult<V> = std::result::Result<V, ServiceError>;

#[derive(Serialize, Deserialize, Debug)]
#[cfg(not(tarpaulin_include))]
pub struct ErrorToResponse {
    pub error: String,
}

#[cfg(not(tarpaulin_include))]
impl ResponseError for ServiceError {
    #[cfg(not(tarpaulin_include))]
    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code())
            .append_header((header::CONTENT_TYPE, "application/json; charset=UTF-8"))
            .body(
                serde_json::to_string(&ErrorToResponse {
                    error: self.to_string(),
                })
                .unwrap(),
            )
    }

    #[cfg(not(tarpaulin_include))]
    fn status_code(&self) -> StatusCode {
        match self {
            ServiceError::ClosedForRegistration => StatusCode::FORBIDDEN, //FORBIDDEN,
            ServiceError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR, // INTERNAL SERVER ERROR
            ServiceError::NotAnEmail => StatusCode::BAD_REQUEST,                    //BADREQUEST,
            ServiceError::NotAUrl => StatusCode::BAD_REQUEST,                       //BADREQUEST,
            ServiceError::NotAnId => StatusCode::BAD_REQUEST,                       //BADREQUEST,
            ServiceError::URLTooLong => StatusCode::BAD_REQUEST,                    //BADREQUEST,
            ServiceError::WrongPassword => StatusCode::UNAUTHORIZED,                //UNAUTHORIZED,
            ServiceError::AccountNotFound => StatusCode::NOT_FOUND,                 //NOT FOUND,

            ServiceError::ProfainityError => StatusCode::BAD_REQUEST, //BADREQUEST,
            ServiceError::BlacklistError => StatusCode::BAD_REQUEST,  //BADREQUEST,
            ServiceError::UsernameCaseMappedError => StatusCode::BAD_REQUEST, //BADREQUEST,

            ServiceError::PasswordTooShort => StatusCode::BAD_REQUEST, //BADREQUEST,
            ServiceError::PasswordTooLong => StatusCode::BAD_REQUEST,  //BADREQUEST,
            ServiceError::PasswordsDontMatch => StatusCode::BAD_REQUEST, //BADREQUEST,

            ServiceError::UsernameTaken => StatusCode::BAD_REQUEST, //BADREQUEST,
            ServiceError::EmailTaken => StatusCode::BAD_REQUEST,    //BADREQUEST,
            ServiceError::FSError(_) => StatusCode::INTERNAL_SERVER_ERROR,

            ServiceError::GistNotFound => StatusCode::NOT_FOUND,
            ServiceError::CommentNotFound => StatusCode::NOT_FOUND,
            ServiceError::EmptyComment => StatusCode::BAD_REQUEST,

            ServiceError::UnauthorizedOperation(_) => StatusCode::UNAUTHORIZED,
            ServiceError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ServiceError::GistEmpty => StatusCode::BAD_REQUEST,
        }
    }
}
