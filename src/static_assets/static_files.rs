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
use std::borrow::Cow;

use actix_web::body::BoxBody;
use actix_web::{get, http::header, web, HttpResponse, Responder};
use mime_guess::from_path;
use rust_embed::RustEmbed;

use crate::CACHE_AGE;

pub mod assets {
    use crate::FILES;
    use lazy_static::lazy_static;

    lazy_static! {
        pub static ref CSS: &'static str = FILES.get("./static/cache/css/main.css").unwrap();
    }
}

#[derive(RustEmbed)]
#[folder = "assets/"]
struct Asset;

fn handle_assets(path: &str) -> HttpResponse {
    match Asset::get(path) {
        Some(content) => {
            let body: BoxBody = match content.data {
                Cow::Borrowed(bytes) => BoxBody::new(bytes),
                Cow::Owned(bytes) => BoxBody::new(bytes),
            };

            HttpResponse::Ok()
                .insert_header(header::CacheControl(vec![
                    header::CacheDirective::Public,
                    header::CacheDirective::Extension("immutable".into(), None),
                    header::CacheDirective::MaxAge(CACHE_AGE),
                ]))
                .content_type(from_path(path).first_or_octet_stream().as_ref())
                .body(body)
        }
        None => HttpResponse::NotFound().body("404 Not Found"),
    }
}

#[get("/assets/{_:.*}")]
pub async fn static_files(path: web::Path<String>) -> impl Responder {
    handle_assets(&path)
}

#[cfg(test)]
mod tests {
    use actix_web::http::StatusCode;
    use actix_web::test;

    use crate::data::Data;
    use crate::db::BoxDB;
    use crate::tests::*;
    use crate::*;

    use super::assets::CSS;

    #[actix_rt::test]
    async fn postgrest_static_files_works() {
        let (db, data) = sqlx_postgres::get_data().await;
        static_assets_work(data, db).await;
    }

    #[actix_rt::test]
    async fn sqlite_static_files_works() {
        let (db, data) = sqlx_sqlite::get_data().await;
        static_assets_work(data, db).await;
    }

    async fn static_assets_work(data: Arc<Data>, db: BoxDB) {
        let app = get_app!(data, db).await;

        let file = *CSS;
        let resp = get_request!(&app, file);
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
