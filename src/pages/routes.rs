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
use actix_auth_middleware::{Authentication, GetLoginRoute};
use serde::*;

pub use crate::api::v1::routes::PostCommentPath;

/// constant [Pages](Pages) instance
pub const PAGES: Pages = Pages::new();

#[derive(Serialize)]
/// Top-level routes data structure for V1 AP1
pub struct Pages {
    /// Authentication routes
    pub auth: Auth,
    /// Gist routes
    pub gist: Gists,
    /// home page
    pub home: &'static str,
}

impl Pages {
    /// create new instance of Routes
    const fn new() -> Pages {
        let gist = Gists::new();
        let home = gist.new;
        Pages {
            auth: Auth::new(),
            gist,
            home,
        }
    }
}

#[derive(Serialize)]
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
        let login = "/login";
        let logout = "/logout";
        let register = "/join";
        Auth {
            logout,
            login,
            register,
        }
    }
}

#[derive(Deserialize)]
pub struct GistProfilePathComponent<'a> {
    pub username: &'a str,
}

#[derive(Serialize)]
/// Gist routes
pub struct Gists {
    /// profile route
    pub profile: &'static str,
    /// new gist route
    pub new: &'static str,
    /// view gist
    pub view_gist: &'static str,
    /// post comment on gist
    pub post_comment: &'static str,
}

impl Gists {
    /// create new instance of Gists route
    pub const fn new() -> Self {
        let profile = "/{username}";
        let view_gist = "/{username}/{gist}";
        let post_comment = "/{username}/{gist}/comment";
        let new = "/";
        Self {
            profile,
            new,
            view_gist,
            post_comment,
        }
    }

    /// get profile route with placeholders replaced with values provided.
    pub fn get_profile_route(&self, components: GistProfilePathComponent) -> String {
        self.profile.replace("{username}", components.username)
    }

    /// get gist route route with placeholders replaced with values provided.
    pub fn get_gist_route(&self, components: &PostCommentPath) -> String {
        self.view_gist
            .replace("{username}", &components.username)
            .replace("{gist}", &components.gist)
    }

    /// get post_comment route with placeholders replaced with values provided.
    pub fn get_post_comment_route(&self, components: &PostCommentPath) -> String {
        self.post_comment
            .replace("{username}", &components.username)
            .replace("{gist}", &components.gist)
    }
}

pub fn get_auth_middleware() -> Authentication<Pages> {
    Authentication::with_identity(PAGES)
}

impl GetLoginRoute for Pages {
    fn get_login_route(&self, src: Option<&str>) -> String {
        if let Some(redirect_to) = src {
            format!(
                "{}?redirect_to={}",
                self.auth.login,
                urlencoding::encode(redirect_to)
            )
        } else {
            self.auth.login.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gist_route_substitution_works() {
        const NAME: &str = "bob";
        const GIST: &str = "foo";
        let get_profile = format!("/{NAME}");
        let view_gist = format!("/{NAME}/{GIST}");
        let post_comment = format!("/{NAME}/{GIST}/comment");

        let profile_component = GistProfilePathComponent { username: NAME };

        assert_eq!(get_profile, PAGES.gist.get_profile_route(profile_component));

        let profile_component = PostCommentPath {
            username: NAME.into(),
            gist: GIST.into(),
        };

        assert_eq!(view_gist, PAGES.gist.get_gist_route(&profile_component));

        let post_comment_path = PostCommentPath {
            gist: GIST.into(),
            username: NAME.into(),
        };

        assert_eq!(
            post_comment,
            PAGES.gist.get_post_comment_route(&post_comment_path)
        );
    }
}
