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
use std::fmt;

use actix_web::{
    error::ResponseError,
    http::{header::ContentType, StatusCode},
    HttpResponse, HttpResponseBuilder,
};
use derive_more::Display;
use derive_more::Error;
use serde::*;

use crate::errors::ServiceError;

pub const ERROR_KEY: &str = "error";
pub const ERROR_PAGE: &str = "error_comp";

pub fn register_templates(t: &mut tera::Tera) {
    if let Err(e) =
        t.add_template_files(vec![("templates/components/error.html", Some(ERROR_PAGE))])
    {
        println!("Parsing error(s): {}", e);
        ::std::process::exit(1);
    };
}

/// Render template with error context
pub trait CtxError {
    fn with_error(&self, e: &ReadableError) -> String;
}

#[derive(Serialize, Debug, Display, Clone)]
#[display(fmt = "title: {} reason: {}", title, reason)]
pub struct ReadableError {
    pub reason: String,
    pub title: String,
}

impl ReadableError {
    pub fn new(e: &ServiceError) -> Self {
        let reason = format!("{}", e);
        let title = format!("{}", e.status_code());

        Self { reason, title }
    }
}

#[derive(Error, Display)]
#[display(fmt = "{}", readable)]
pub struct PageError<T> {
    #[error(not(source))]
    template: T,
    readable: ReadableError,
    #[error(not(source))]
    error: ServiceError,
}

impl<T> fmt::Debug for PageError<T> {
    #[cfg(not(tarpaulin_include))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PageError")
            .field("readable", &self.readable)
            .finish()
    }
}

impl<T: CtxError> PageError<T> {
    /// create new instance of [PageError] from a template and an error
    pub fn new(template: T, error: ServiceError) -> Self {
        let readable = ReadableError::new(&error);
        Self {
            error,
            template,
            readable,
        }
    }
}

#[cfg(not(tarpaulin_include))]
impl<T: CtxError> ResponseError for PageError<T> {
    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code())
            .content_type(ContentType::html())
            .body(self.template.with_error(&self.readable))
    }

    fn status_code(&self) -> StatusCode {
        self.error.status_code()
    }
}

/// Generic result data structure
#[cfg(not(tarpaulin_include))]
pub type PageResult<V, T> = std::result::Result<V, PageError<T>>;
