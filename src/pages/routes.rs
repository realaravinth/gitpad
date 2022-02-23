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
pub struct GistProfilePath {
    pub username: String,
}

#[derive(Serialize)]
/// Gist routes
pub struct Gists {
    /// logout route
    pub profile: &'static str,
    /// login route
    pub new: &'static str,
}

impl Gists {
    /// create new instance of Gists route
    pub const fn new() -> Self {
        let profile = "/{username}";
        let new = "/";
        Self { profile, new }
    }

    /// get profile route with placeholders replaced with values provided.
    pub fn get_profile_route(&self, components: &GistProfilePath) -> String {
        self.profile.replace("{username}", &components.username)
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
            self.auth.register.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gist_route_substitution_works() {
        const NAME: &str = "bob";
        let get_profile = format!("/{NAME}");

        let profile_component = GistProfilePath {
            username: NAME.into(),
        };

        assert_eq!(
            get_profile,
            PAGES.gist.get_profile_route(&profile_component)
        );
    }
}
