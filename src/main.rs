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
use std::env;
use std::sync::Arc;

use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{
    error::InternalError, http::StatusCode, middleware as actix_middleware, web::Data as WebData,
    web::JsonConfig, App, HttpServer,
};
use log::info;

mod api;
pub mod data;
mod db;
pub mod demo;
pub mod errors;
mod routes;
mod settings;
#[cfg(test)]
mod tests;
mod utils;

pub use api::v1::ROUTES as V1_API_ROUTES;
pub use data::Data;
pub use settings::Settings;

pub const CACHE_AGE: u32 = 604800;

pub const GIT_COMMIT_HASH: &str = env!("GIT_HASH");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");
pub const PKG_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
pub const PKG_HOMEPAGE: &str = env!("CARGO_PKG_HOMEPAGE");

pub type AppData = WebData<Arc<data::Data>>;
pub type DB = WebData<Box<dyn db_core::GPDatabse>>;

#[cfg(not(tarpaulin_include))]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let settings = Settings::new().unwrap();
    pretty_env_logger::init();

    info!(
        "{}: {}.\nFor more information, see: {}\nBuild info:\nVersion: {} commit: {}",
        PKG_NAME,
        PKG_DESCRIPTION,
        PKG_HOMEPAGE,
        VERSION,
        &GIT_COMMIT_HASH[..10]
    );

    println!("Starting server on: http://{}", settings.server.get_ip());
    let workers = settings.server.workers.unwrap_or_else(num_cpus::get);
    let socket_addr = settings.server.get_ip();

    log::info!("DB type: {}", settings.database.database_type);
    let db = match settings.database.database_type {
        settings::DBType::Sqlite => db::sqlite::get_data(Some(settings.clone())).await,
        settings::DBType::Postgres => db::pg::get_data(Some(settings.clone())).await,
    };
    let db = WebData::new(db);

    let data = WebData::new(data::Data::new(Some(settings)));
    HttpServer::new(move || {
        App::new()
            .wrap(actix_middleware::Logger::default())
            .wrap(actix_middleware::Compress::default())
            .app_data(data.clone())
            .app_data(db.clone())
            .app_data(get_json_err())
            .wrap(
                actix_middleware::DefaultHeaders::new()
                    .add(("Permissions-Policy", "interest-cohort=()")),
            )
            .wrap(actix_middleware::NormalizePath::new(
                actix_middleware::TrailingSlash::Trim,
            ))
            .configure(routes::services)
    })
    .workers(workers)
    .bind(socket_addr)
    .unwrap()
    .run()
    .await
}

#[cfg(not(tarpaulin_include))]
pub fn get_json_err() -> JsonConfig {
    JsonConfig::default()
        .error_handler(|err, _| InternalError::new(err, StatusCode::BAD_REQUEST).into())
}

#[cfg(not(tarpaulin_include))]
pub fn get_identity_service(settings: &Settings) -> IdentityService<CookieIdentityPolicy> {
    let cookie_secret = &settings.server.cookie_secret;
    IdentityService::new(
        CookieIdentityPolicy::new(cookie_secret.as_bytes())
            .name("Authorization")
            //TODO change cookie age
            .max_age_secs(216000)
            .domain(&settings.server.domain)
            .secure(false),
    )
}
