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
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use crate::{DB, GIT_COMMIT_HASH, VERSION};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BuildDetails {
    pub version: &'static str,
    pub git_commit_hash: &'static str,
}

pub mod routes {
    pub struct Meta {
        pub build_details: &'static str,
        pub health: &'static str,
    }

    impl Meta {
        pub const fn new() -> Self {
            Self {
                build_details: "/api/v1/meta/build",
                health: "/api/v1/meta/health",
            }
        }
    }
}

/// emmits build details of the bninary
#[my_codegen::get(path = "crate::V1_API_ROUTES.meta.build_details")]
async fn build_details() -> impl Responder {
    let build = BuildDetails {
        version: VERSION,
        git_commit_hash: GIT_COMMIT_HASH,
    };
    HttpResponse::Ok().json(build)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
/// Health check return datatype
pub struct Health {
    db: bool,
}

/// emmits build details of the bninary
#[my_codegen::get(path = "crate::V1_API_ROUTES.meta.health")]
async fn health(db: DB) -> impl Responder {
    let health = Health {
        db: db.ping().await,
    };
    HttpResponse::Ok().json(health)
}

pub fn services(cfg: &mut web::ServiceConfig) {
    cfg.service(health);
    cfg.service(build_details);
}

#[cfg(test)]
mod tests {
    use actix_web::{http::StatusCode, test, App};

    use crate::api::v1::meta::Health;
    use crate::routes::services;
    use crate::tests::*;
    use crate::*;

    #[actix_rt::test]
    async fn build_details_works() {
        let app = test::init_service(App::new().configure(services)).await;

        let resp = test::call_service(
            &app,
            test::TestRequest::get()
                .uri(V1_API_ROUTES.meta.build_details)
                .to_request(),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_rt::test]
    async fn health_works() {
        let config = [
            sqlx_postgres::get_data().await,
            sqlx_sqlite::get_data().await,
        ];

        for (db, data) in config.iter() {
            let app = get_app!(data, db).await;
            let resp = get_request!(&app, &V1_API_ROUTES.meta.health);
            assert_eq!(resp.status(), StatusCode::OK);

            let health: Health = test::read_body_json(resp).await;
            assert!(health.db);
        }
    }
}
