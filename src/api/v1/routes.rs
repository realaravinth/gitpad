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
//! V1 API Routes
use actix_auth_middleware::{Authentication, GetLoginRoute};
use serde::*;

use super::meta::routes::Meta;

/// constant [Routes](Routes) instance
pub const ROUTES: Routes = Routes::new();

/// Authentication routes
pub struct Auth {
    /// logout route
    pub logout: &'static str,
    /// login route
    pub login: &'static str,
    /// registration route
    pub register: &'static str,
}
impl Auth {
    /// create new instance of Authentication route
    pub const fn new() -> Auth {
        let login = "/api/v1/signin";
        let logout = "/logout";
        let register = "/api/v1/signup";
        Auth {
            logout,
            login,
            register,
        }
    }
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetFilePath {
    pub username: String,
    pub gist: String,
    pub file: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PostCommentPath {
    pub username: String,
    pub gist: String,
}

/// Authentication routes
pub struct Gist {
    /// logout route
    pub new: &'static str,

    /// get flie route
    pub get_file: &'static str,

    /// post comment on gist
    pub post_comment: &'static str,
}

impl Gist {
    /// create new instance of Authentication route
    pub const fn new() -> Gist {
        let new = "/api/v1/gist/new";
        let get_file = "/api/v1/gist/profile/{username}/{gist}/contents/{file}";
        let post_comment = "/api/v1/gist/profile/{username}/{gist}/comments";
        Gist {
            new,
            get_file,
            post_comment,
        }
    }

    /// get file routes with placeholders replaced with values provided.
    /// filename is auto-escaped using [urlencoding::encode]
    pub fn get_file_route(&self, components: &GetFilePath) -> String {
        self.get_file
            .replace("{username}", &components.username)
            .replace("{gist}", &components.gist)
            .replace("{file}", &urlencoding::encode(&components.file))
    }

    /// get post_comment route with placeholders replaced with values provided.
    pub fn get_post_comment_route(&self, components: &PostCommentPath) -> String {
        self.post_comment
            .replace("{username}", &components.username)
            .replace("{gist}", &components.gist)
    }
}

/// Account management routes
pub struct Account {
    /// delete account route
    pub delete: &'static str,
    /// route to check if an email exists
    pub email_exists: &'static str,
    /// route to fetch account secret
    pub get_secret: &'static str,
    /// route to update a user's email
    pub update_email: &'static str,
    ///    route to update password
    pub update_password: &'static str,
    ///    route to update secret
    pub update_secret: &'static str,
    ///    route to check if a username is already registered
    pub username_exists: &'static str,
    ///    route to change username
    pub update_username: &'static str,
}

impl Account {
    /// create a new instance of [Account][Account] routes
    pub const fn new() -> Account {
        let get_secret = "/api/v1/account/secret/get";
        let update_secret = "/api/v1/account/secret/update";
        let delete = "/api/v1/account/delete";
        let email_exists = "/api/v1/account/email/exists";
        let username_exists = "/api/v1/account/username/exists";
        let update_username = "/api/v1/account/username/update";
        let update_email = "/api/v1/account/email/update";
        let update_password = "/api/v1/account/password/update";
        Account {
            delete,
            email_exists,
            get_secret,
            update_email,
            update_password,
            update_secret,
            username_exists,
            update_username,
        }
    }
}

/// Top-level routes data structure for V1 AP1
pub struct Routes {
    /// Authentication routes
    pub auth: Auth,
    /// Account routes
    pub account: Account,
    /// Meta routes
    pub meta: Meta,
    /// Gist routes
    pub gist: Gist,
}

impl Routes {
    /// create new instance of Routes
    const fn new() -> Routes {
        Routes {
            auth: Auth::new(),
            account: Account::new(),
            meta: Meta::new(),
            gist: Gist::new(),
        }
    }
}

pub fn get_auth_middleware() -> Authentication<Routes> {
    Authentication::with_identity(ROUTES)
}

impl GetLoginRoute for Routes {
    fn get_login_route(&self, src: Option<&str>) -> String {
        if let Some(redirect_to) = src {
            format!(
                "{}?redirect_to={}",
                self.auth.login,
                urlencoding::encode(redirect_to)
            )
        } else {
            self.auth.register.to_string()
        }
    }
}
