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
use serde::*;
use tera::*;

use crate::settings::Settings;
use crate::static_assets::ASSETS;
use crate::{GIT_COMMIT_HASH, VERSION};

pub mod auth;
pub mod routes;

pub use routes::get_auth_middleware;
pub use routes::PAGES;

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = Tera::default();
        if let Err(e) = tera.add_template_files(vec![
            ("templates/components/base.html", Some("base")),
            ("templates/components/footer.html", Some("footer")),
            ("templates/components/nav.html", Some("nav")),
        ]) {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        };
        tera.autoescape_on(vec![".html", ".sql"]);
        auth::register_templates(&mut tera);
        tera
    };
}

pub fn context(s: &Settings) -> Context {
    let mut ctx = Context::new();
    let footer = Footer::new(s);
    ctx.insert("footer", &footer);
    ctx.insert("page", &PAGES);
    ctx.insert("assets", &*ASSETS);
    ctx
}

pub fn init(s: &Settings) {
    auth::login::Login::page(s);
    auth::register::Register::page(s);
}

#[derive(Serialize)]
pub struct Footer<'a> {
    version: &'a str,
    admin_email: &'a str,
    source_code: &'a str,
    git_hash: &'a str,
}

impl<'a> Footer<'a> {
    pub fn new(settings: &'a Settings) -> Self {
        Self {
            version: VERSION,
            source_code: &settings.source_code,
            admin_email: &settings.admin_email,
            git_hash: &GIT_COMMIT_HASH[..8],
        }
    }
}

pub fn services(cfg: &mut web::ServiceConfig) {
    auth::services(cfg);
}

#[cfg(test)]
mod tests {
    use super::{init, Settings};

    #[test]
    fn templates_work() {
        let settings = Settings::new().unwrap();
        init(&settings);
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
