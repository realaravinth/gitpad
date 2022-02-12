/*
 * Copyright (C) 2021  Aravinth Manivannan <realaravinth@batsense.net>
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
use std::sync::Arc;

use actix_auth_middleware::GetLoginRoute;
use actix_web::http::{header, StatusCode};
use actix_web::test;

use crate::api::v1::ROUTES;
use crate::data::api::v1::auth::{Login, Register};
use crate::data::Data;
use crate::db::BoxDB;
use crate::errors::*;
use crate::*;

use crate::tests::*;

#[actix_rt::test]
async fn postgrest_auth_works() {
    let (db, data) = sqlx_postgres::get_data().await;
    auth_works(data.clone(), db.clone()).await;
    serverside_password_validation_works(data, db).await;
}

#[actix_rt::test]
async fn sqlite_auth_works() {
    let (db, data) = sqlx_sqlite::get_data().await;
    auth_works(data.clone(), db.clone()).await;
    serverside_password_validation_works(data, db).await;
}

async fn auth_works(data: Arc<Data>, db: BoxDB) {
    const NAME: &str = "testuserfoo";
    const PASSWORD: &str = "longpassword";
    const EMAIL: &str = "testuser1foo@a.com";

    let _ = data.delete_user(&db, NAME, PASSWORD).await;
    let app = get_app!(data, db).await;

    // 1. Register with email == None
    let msg = Register {
        username: NAME.into(),
        password: PASSWORD.into(),
        confirm_password: PASSWORD.into(),
        email: None,
    };
    let resp =
        test::call_service(&app, post_request!(&msg, ROUTES.auth.register).to_request()).await;
    assert_eq!(resp.status(), StatusCode::OK);
    // delete user TODO
    let _ = data.delete_user(&db, NAME, PASSWORD).await;

    // 1. Register and signin
    let (_, signin_resp) = data.register_and_signin(&db, NAME, EMAIL, PASSWORD).await;
    let cookies = get_cookie!(signin_resp);

    // Sign in with email
    data.signin_test(&db, EMAIL, PASSWORD).await;

    // 2. check if duplicate username is allowed
    let mut msg = Register {
        username: NAME.into(),
        password: PASSWORD.into(),
        confirm_password: PASSWORD.into(),
        email: Some(EMAIL.into()),
    };

    msg.username = format!("asdfasd{}", msg.username);
    data.bad_post_req_test(
        &db,
        NAME,
        PASSWORD,
        ROUTES.auth.register,
        &msg,
        ServiceError::EmailTaken,
    )
    .await;

    msg.email = Some(format!("asdfasd{}", msg.email.unwrap()));
    msg.username = NAME.into();
    data.bad_post_req_test(
        &db,
        NAME,
        PASSWORD,
        ROUTES.auth.register,
        &msg,
        ServiceError::UsernameTaken,
    )
    .await;

    // 3. sigining in with non-existent user
    let mut creds = Login {
        login: "nonexistantuser".into(),
        password: msg.password.clone(),
    };
    data.bad_post_req_test(
        &db,
        NAME,
        PASSWORD,
        ROUTES.auth.login,
        &creds,
        ServiceError::AccountNotFound,
    )
    .await;

    creds.login = "nonexistantuser@example.com".into();
    data.bad_post_req_test(
        &db,
        NAME,
        PASSWORD,
        ROUTES.auth.login,
        &creds,
        ServiceError::AccountNotFound,
    )
    .await;

    // 4. trying to signin with wrong password
    creds.login = NAME.into();
    creds.password = NAME.into();

    data.bad_post_req_test(
        &db,
        NAME,
        PASSWORD,
        ROUTES.auth.login,
        &creds,
        ServiceError::WrongPassword,
    )
    .await;

    // 5. signout
    let signout_resp = test::call_service(
        &app,
        test::TestRequest::get()
            .uri(ROUTES.auth.logout)
            .cookie(cookies)
            .to_request(),
    )
    .await;
    assert_eq!(signout_resp.status(), StatusCode::FOUND);
    let headers = signout_resp.headers();
    assert_eq!(
        headers.get(header::LOCATION).unwrap(),
        &crate::V1_API_ROUTES.get_login_route(None)
    );

    let creds = Login {
        login: NAME.into(),
        password: PASSWORD.into(),
    };

    //6. sigin with redirect URL set
    let redirect_to = ROUTES.auth.logout;
    let resp = test::call_service(
        &app,
        post_request!(&creds, &ROUTES.get_login_route(Some(redirect_to))).to_request(),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::FOUND);
    let headers = resp.headers();
    assert_eq!(headers.get(header::LOCATION).unwrap(), &redirect_to);
}

async fn serverside_password_validation_works(data: Arc<Data>, db: BoxDB) {
    const NAME: &str = "testuser542";
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
        post_request!(&register_msg, ROUTES.auth.register).to_request(),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let txt: ErrorToResponse = test::read_body_json(resp).await;
    assert_eq!(txt.error, format!("{}", ServiceError::PasswordsDontMatch));
}
