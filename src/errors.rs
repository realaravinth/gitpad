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

use argon2_creds::errors::CredsError;
use db_core::errors::DBError;
use derive_more::{Display, Error};
use serde::{Deserialize, Serialize};
use url::ParseError;
use validator::ValidationErrors;

#[derive(Debug, Display, Error)]
pub struct FSError(#[display(fmt = "File System Error {}", _0)] pub FSErrorInner);

impl PartialEq for FSError {
    fn eq(&self, other: &Self) -> bool {
        self.0.kind() == other.0.kind()
    }
}

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

use actix_web::{
    error::ResponseError,
    http::{header, StatusCode},
    HttpResponse, HttpResponseBuilder,
};

#[derive(Serialize, Deserialize)]
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
        let status_code = match self {
            ServiceError::ClosedForRegistration => 403, //FORBIDDEN,
            ServiceError::InternalServerError => 500,   // INTERNAL SERVER ERROR
            ServiceError::NotAnEmail => 400,            //BADREQUEST,
            ServiceError::NotAUrl => 400,               //BADREQUEST,
            ServiceError::NotAnId => 400,               //BADREQUEST,
            ServiceError::URLTooLong => 400,            //BADREQUEST,
            ServiceError::WrongPassword => 401,         //UNAUTHORIZED,
            ServiceError::AccountNotFound => 404,       //NOT FOUND,

            ServiceError::ProfainityError => 400, //BADREQUEST,
            ServiceError::BlacklistError => 400,  //BADREQUEST,
            ServiceError::UsernameCaseMappedError => 400, //BADREQUEST,

            ServiceError::PasswordTooShort => 400, //BADREQUEST,
            ServiceError::PasswordTooLong => 400,  //BADREQUEST,
            ServiceError::PasswordsDontMatch => 400, //BADREQUEST,

            ServiceError::UsernameTaken => 400, //BADREQUEST,
            ServiceError::EmailTaken => 400,    //BADREQUEST,
            ServiceError::FSError(_) => 500,

            ServiceError::GistNotFound => 404,
            ServiceError::CommentNotFound => 404,
        };

        StatusCode::from_u16(status_code).unwrap()
    }
}
