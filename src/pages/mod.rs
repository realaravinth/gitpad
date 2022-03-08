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
use actix_web::*;
use lazy_static::lazy_static;
use rust_embed::RustEmbed;
use serde::*;
use tera::*;

use crate::settings::Settings;
use crate::static_assets::ASSETS;
use crate::{GIT_COMMIT_HASH, VERSION};

pub mod auth;
pub mod errors;
pub mod gists;
pub mod routes;

pub use routes::get_auth_middleware;
pub use routes::PAGES;

pub struct TemplateFile {
    pub name: &'static str,
    pub path: &'static str,
}

impl TemplateFile {
    pub const fn new(name: &'static str, path: &'static str) -> Self {
        Self { name, path }
    }

    pub fn register(&self, t: &mut Tera) -> std::result::Result<(), tera::Error> {
        t.add_raw_template(self.name, &Templates::get_template(self).expect(self.name))
    }

    #[cfg(test)]
    #[allow(dead_code)]
    pub fn register_from_file(&self, t: &mut Tera) -> std::result::Result<(), tera::Error> {
        use std::path::Path;
        t.add_template_file(Path::new("templates/").join(self.path), Some(self.name))
    }
}

pub const PAYLOAD_KEY: &str = "payload";

pub const BASE: TemplateFile = TemplateFile::new("base", "components/base.html");
pub const FOOTER: TemplateFile = TemplateFile::new("footer", "components/footer.html");
pub const PUB_NAV: TemplateFile = TemplateFile::new("pub_nav", "components/nav/pub.html");
pub const AUTH_NAV: TemplateFile = TemplateFile::new("auth_nav", "components/nav/auth.html");

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = Tera::default();
        for t in [BASE, FOOTER, PUB_NAV, AUTH_NAV].iter() {
            t.register(&mut tera).unwrap();
        }
        errors::register_templates(&mut tera);
        tera.autoescape_on(vec![".html", ".sql"]);
        auth::register_templates(&mut tera);
        gists::register_templates(&mut tera);
        tera
    };
}

#[derive(RustEmbed)]
#[folder = "templates/"]
pub struct Templates;

impl Templates {
    pub fn get_template(t: &TemplateFile) -> Option<String> {
        match Self::get(t.path) {
            Some(file) => Some(String::from_utf8_lossy(&file.data).into_owned()),
            None => None,
        }
    }
}

pub fn context(s: &Settings) -> Context {
    let mut ctx = Context::new();
    let footer = Footer::new(s);
    ctx.insert("footer", &footer);
    ctx.insert("page", &PAGES);
    ctx.insert("assets", &*ASSETS);
    ctx
}

pub fn auth_ctx(username: &str, s: &Settings) -> Context {
    use routes::GistProfilePathComponent;
    let profile_link = PAGES
        .gist
        .get_profile_route(GistProfilePathComponent { username });
    let mut ctx = Context::new();
    let footer = Footer::new(s);
    ctx.insert("footer", &footer);
    ctx.insert("page", &PAGES);
    ctx.insert("assets", &*ASSETS);
    ctx.insert("loggedin_user", &profile_link);
    ctx
}

#[derive(Serialize)]
pub struct Footer<'a> {
    version: &'a str,
    admin_email: &'a str,
    source_code: &'a str,
    git_hash: &'a str,
    settings: &'a Settings,
    demo_user: &'a str,
    demo_password: &'a str,
}

impl<'a> Footer<'a> {
    pub fn new(settings: &'a Settings) -> Self {
        Self {
            version: VERSION,
            source_code: &settings.source_code,
            admin_email: &settings.admin_email,
            git_hash: &GIT_COMMIT_HASH[..8],
            demo_user: crate::demo::DEMO_USER,
            demo_password: crate::demo::DEMO_PASSWORD,
            settings,
        }
    }
}

pub fn services(cfg: &mut web::ServiceConfig) {
    auth::services(cfg);
    gists::services(cfg);
}

#[cfg(test)]
mod tests {

    #[test]
    fn templates_work_basic() {
        use super::*;
        use tera::Tera;

        let mut tera = Tera::default();
        let mut tera2 = Tera::default();
        for t in [
            BASE,
            FOOTER,
            PUB_NAV,
            AUTH_NAV,
            auth::AUTH_BASE,
            auth::login::LOGIN,
            auth::register::REGISTER,
            errors::ERROR_TEMPLATE,
            gists::GIST_BASE,
            gists::GIST_EXPLORE,
            gists::new::NEW_GIST,
        ]
        .iter()
        {
            t.register_from_file(&mut tera2).unwrap();
            t.register(&mut tera).unwrap();
        }
    }
}

#[cfg(test)]
mod http_page_tests {
    use actix_web::http::StatusCode;
    use actix_web::test;

    use crate::data::Data;
    use crate::db::BoxDB;
    use crate::tests::*;
    use crate::*;

    use super::PAGES;

    #[actix_rt::test]
    async fn postgrest_templates_work() {
        let (db, data) = sqlx_postgres::get_data().await;
        templates_work(data, db).await;
    }

    #[actix_rt::test]
    async fn sqlite_templates_work() {
        let (db, data) = sqlx_sqlite::get_data().await;
        templates_work(data, db).await;
    }

    async fn templates_work(data: Arc<Data>, db: BoxDB) {
        let app = get_app!(data, db).await;

        for file in [PAGES.auth.login, PAGES.auth.register].iter() {
            let resp = get_request!(&app, file);
            assert_eq!(resp.status(), StatusCode::OK);
        }
    }
}
