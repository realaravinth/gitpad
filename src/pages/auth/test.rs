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
use actix_auth_middleware::GetLoginRoute;

use actix_web::http::header;
use actix_web::http::StatusCode;
use actix_web::test;

use super::*;

use crate::data::api::v1::auth::{Login, Register};
use crate::data::Data;
use crate::errors::*;
use crate::tests::*;
use crate::*;

#[actix_rt::test]
async fn postgrest_pages_auth_works() {
    let (db, data) = sqlx_postgres::get_data().await;
    auth_works(data.clone(), db.clone()).await;
    serverside_password_validation_works(data.clone(), db.clone()).await;
}

#[actix_rt::test]
async fn sqlite_pages_auth_works() {
    let (db, data) = sqlx_sqlite::get_data().await;
    auth_works(data.clone(), db.clone()).await;
    serverside_password_validation_works(data.clone(), db.clone()).await;
}

async fn auth_works(data: Arc<Data>, db: BoxDB) {
    const NAME: &str = "testuserform";
    const EMAIL: &str = "testuserform@foo.com";
    const PASSWORD: &str = "longpassword";

    let _ = data.delete_user(&db, NAME, PASSWORD).await;
    let app = get_app!(data, db).await;

    // 1. Register with email == None
    let msg = Register {
        username: NAME.into(),
        password: PASSWORD.into(),
        confirm_password: PASSWORD.into(),
        email: None,
    };
    let resp = test::call_service(
        &app,
        post_request!(&msg, PAGES.auth.register, FORM).to_request(),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::FOUND);
    let headers = resp.headers();
    assert_eq!(headers.get(header::LOCATION).unwrap(), PAGES.auth.login);
    let _ = data.delete_user(&db, NAME, PASSWORD).await;

    // 1. Register with email == "" // Form request handler converts empty emails to None
    let msg = Register {
        username: NAME.into(),
        password: PASSWORD.into(),
        confirm_password: PASSWORD.into(),
        email: Some("".into()),
    };
    let resp = test::call_service(
        &app,
        post_request!(&msg, PAGES.auth.register, FORM).to_request(),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::FOUND);
    let headers = resp.headers();
    assert_eq!(headers.get(header::LOCATION).unwrap(), PAGES.auth.login);
    let _ = data.delete_user(&db, NAME, PASSWORD).await;

    // 1. Register with email
    let msg = Register {
        username: NAME.into(),
        password: PASSWORD.into(),
        confirm_password: PASSWORD.into(),
        email: Some(EMAIL.into()),
    };
    let resp = test::call_service(
        &app,
        post_request!(&msg, PAGES.auth.register, FORM).to_request(),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::FOUND);
    let headers = resp.headers();
    assert_eq!(headers.get(header::LOCATION).unwrap(), PAGES.auth.login);

    // sign in
    let msg = Login {
        login: NAME.into(),
        password: PASSWORD.into(),
    };
    let resp = test::call_service(
        &app,
        post_request!(&msg, PAGES.auth.login, FORM).to_request(),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::FOUND);
    let headers = resp.headers();
    assert_eq!(headers.get(header::LOCATION).unwrap(), PAGES.home);

    // redirect after signin
    let redirect = "/foo/bar/nonexistantuser";
    let url = PAGES.get_login_route(Some(redirect));
    let resp = test::call_service(&app, post_request!(&msg, &url, FORM).to_request()).await;
    assert_eq!(resp.status(), StatusCode::FOUND);
    let headers = resp.headers();
    assert_eq!(headers.get(header::LOCATION).unwrap(), &redirect);

    // wrong password signin
    let msg = Login {
        login: NAME.into(),
        password: NAME.into(),
    };
    let resp = test::call_service(
        &app,
        post_request!(&msg, PAGES.auth.login, FORM).to_request(),
    )
    .await;
    assert_eq!(resp.status(), ServiceError::WrongPassword.status_code());
}

async fn serverside_password_validation_works(data: Arc<Data>, db: BoxDB) {
    const NAME: &str = "pagetestuser542";
    const PASSWORD: &str = "longpassword2";

    let db = &db;
    let _ = data.delete_user(db, NAME, PASSWORD).await;

    let app = get_app!(data, db).await;

    // checking to see if server-side password validation (password == password_config)
    // works
    let register_msg = Register {
        username: NAME.into(),
        password: PASSWORD.into(),
        confirm_password: NAME.into(),
        email: None,
    };
    let resp = test::call_service(
        &app,
        post_request!(&register_msg, PAGES.auth.register, FORM).to_request(),
    )
    .await;
    assert_eq!(
        resp.status(),
        ServiceError::PasswordsDontMatch.status_code()
    );
}
