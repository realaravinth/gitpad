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
pub use std::sync::Arc;

//use actix_web::cookie::Cookie;
use actix_web::test;
use actix_web::{
    body::{BoxBody, EitherBody},
    dev::ServiceResponse,
    error::ResponseError,
    http::StatusCode,
};
use serde::Serialize;

use crate::api::v1::ROUTES;
use crate::data::api::v1::auth::{Login, Register};
use crate::data::Data;
pub use crate::db::BoxDB;
use crate::errors::*;
use crate::settings::DBType;
use crate::*;
use db_core::prelude::*;

pub mod sqlx_postgres {
    use super::*;
    use db_sqlx_postgres::{ConnectionOptions, Fresh};
    use sqlx::postgres::PgPoolOptions;

    pub async fn get_data() -> (BoxDB, Arc<Data>) {
        let url = env::var("POSTGRES_DATABASE_URL").unwrap();
        let mut settings = Settings::new().unwrap();
        settings.database.url = url.clone();
        settings.database.database_type = DBType::Postgres;
        let pool_options = PgPoolOptions::new().max_connections(2);
        let connection_options = ConnectionOptions::Fresh(Fresh { pool_options, url });

        let db = Box::new(connection_options.connect().await.unwrap());

        (db, Data::new(Some(settings)))
    }
}

pub mod sqlx_sqlite {
    use super::*;
    use db_sqlx_sqlite::{ConnectionOptions, Fresh};
    use sqlx::sqlite::SqlitePoolOptions;

    pub async fn get_data() -> (BoxDB, Arc<Data>) {
        let url = env::var("SQLITE_DATABASE_URL").unwrap();
        let mut settings = Settings::new().unwrap();
        settings.database.url = url.clone();
        settings.database.database_type = DBType::Sqlite;

        let pool_options = SqlitePoolOptions::new().max_connections(2);
        let connection_options = ConnectionOptions::Fresh(Fresh { pool_options, url });

        let db = Box::new(connection_options.connect().await.unwrap());
        (db, Data::new(Some(settings)))
    }
}

#[macro_export]
macro_rules! get_cookie {
    ($resp:expr) => {
        $resp.response().cookies().next().unwrap().to_owned()
    };
}

#[allow(dead_code, clippy::upper_case_acronyms)]
pub struct FORM;

#[macro_export]
macro_rules! post_request {
    ($uri:expr) => {
        test::TestRequest::post().uri($uri)
    };

    ($serializable:expr, $uri:expr) => {
        test::TestRequest::post()
            .uri($uri)
            .insert_header((actix_web::http::header::CONTENT_TYPE, "application/json"))
            .set_payload(serde_json::to_string($serializable).unwrap())
    };

    ($serializable:expr, $uri:expr, FORM) => {
        test::TestRequest::post().uri($uri).set_form($serializable)
    };
}

#[macro_export]
macro_rules! get_request {
    ($app:expr,$route:expr ) => {
        test::call_service(&$app, test::TestRequest::get().uri($route).to_request()).await
    };

    ($app:expr, $route:expr, $cookies:expr) => {
        test::call_service(
            &$app,
            test::TestRequest::get()
                .uri($route)
                .cookie($cookies)
                .to_request(),
        )
        .await
    };
}

#[macro_export]
macro_rules! get_app {
    ("APP", $settings:expr) => {
        actix_web::App::new()
            .app_data(crate::get_json_err())
            .wrap(crate::get_identity_service($settings))
            .wrap(actix_web::middleware::NormalizePath::new(
                actix_web::middleware::TrailingSlash::Trim,
            ))
            .configure(crate::routes::services)
    };

    ($settings:ident) => {
        test::init_service(get_app!("APP", $settings))
    };
    ($data:expr, $db:expr) => {
        test::init_service(
            get_app!("APP", &$data.settings)
                .app_data(crate::DB::new($db.clone()))
                .app_data(crate::AppData::new($data.clone())),
        )
    };
}

impl Data {
    /// register and signin utility
    pub async fn register_and_signin(
        &self,
        db: &BoxDB,
        name: &str,
        email: &str,
        password: &str,
    ) -> (Login, ServiceResponse<EitherBody<BoxBody>>) {
        self.register_test(db, name, email, password).await;
        self.signin_test(&db, name, password).await
    }

    pub fn to_arc(&self) -> Arc<Self> {
        Arc::new(self.clone())
    }

    /// register utility
    pub async fn register_test(&self, db: &BoxDB, name: &str, email: &str, password: &str) {
        let app = get_app!(self.to_arc(), db.clone()).await;

        // 1. Register
        let msg = Register {
            username: name.into(),
            password: password.into(),
            confirm_password: password.into(),
            email: Some(email.into()),
        };
        let resp =
            test::call_service(&app, post_request!(&msg, ROUTES.auth.register).to_request()).await;
        //        let resp_err: ErrorToResponse = test::read_body_json(resp).await;
        //        panic!("{}", resp_err.error);
        assert_eq!(resp.status(), StatusCode::OK);
    }

    /// signin util
    pub async fn signin_test(
        &self,
        db: &BoxDB,
        name: &str,
        password: &str,
    ) -> (Login, ServiceResponse<EitherBody<BoxBody>>) {
        let app = get_app!(self.to_arc(), db.clone()).await;

        // 2. signin
        let creds = Login {
            login: name.into(),
            password: password.into(),
        };
        let signin_resp =
            test::call_service(&app, post_request!(&creds, ROUTES.auth.login).to_request()).await;
        assert_eq!(signin_resp.status(), StatusCode::OK);
        (creds, signin_resp)
    }

    /// pub duplicate test
    pub async fn bad_post_req_test<T: Serialize>(
        &self,
        db: &BoxDB,
        name: &str,
        password: &str,
        url: &str,
        payload: &T,
        err: ServiceError,
    ) {
        let (_, signin_resp) = self.signin_test(db, name, password).await;
        let cookies = get_cookie!(signin_resp);
        let app = get_app!(self.to_arc(), db.clone()).await;

        let resp = test::call_service(
            &app,
            post_request!(&payload, url)
                .cookie(cookies.clone())
                .to_request(),
        )
        .await;
        assert_eq!(resp.status(), err.status_code());
        let resp_err: ErrorToResponse = test::read_body_json(resp).await;
        //println!("{}", txt.error);
        assert_eq!(resp_err.error, format!("{}", err));
    }

    /// bad post req test without payload
    pub async fn bad_post_req_test_witout_payload(
        &self,
        db: &BoxDB,
        name: &str,
        password: &str,
        url: &str,
        err: ServiceError,
    ) {
        let (_, signin_resp) = self.signin_test(db, name, password).await;
        let app = get_app!(self.to_arc(), db.clone()).await;
        let cookies = get_cookie!(signin_resp);

        let resp = test::call_service(
            &app,
            post_request!(url).cookie(cookies.clone()).to_request(),
        )
        .await;
        assert_eq!(resp.status(), err.status_code());
        let resp_err: ErrorToResponse = test::read_body_json(resp).await;
        //println!("{}", resp_err.error);
        assert_eq!(resp_err.error, format!("{}", err));
    }
}
