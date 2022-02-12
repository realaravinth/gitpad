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
use std::sync::Arc;

use db_core::prelude::*;

use crate::api::v1::account::ChangePasswordReqest;
use crate::api::v1::auth::Register;
use crate::errors::*;
use crate::tests::*;
use crate::*;

#[actix_rt::test]
async fn postgrest_account_works() {
    let (db, data) = sqlx_postgres::get_data().await;
    uname_email_exists_works(data, db).await;
    email_udpate_password_validation_del_userworks(data,db).await;
    username_update_works(data, db).await;
}

#[actix_rt::test]
async fn sqlite_account_works() {
    let (db, data) = sqlx_sqlite::get_data().await;
    uname_email_exists_works(data, db).await;
    email_udpate_password_validation_del_userworks(data,db).await;
    username_update_works(data, db).await;
}

async fn uname_email_exists_works(data: Arc<Data>, db: DB) {
    const NAME: &str = "testuserexists";
    const NAME2: &str = "testuserexists2";
    const NAME3: &str = "testuserexists3";
    const PASSWORD: &str = "longpassword2";
    const EMAIL: &str = "accotestsuser@a.com";
    const EMAIL2: &str = "accotestsuser2@a.com";
    const EMAIL3: &str = "accotestsuser3@a.com";
    let db = &db;

    let _ = data.delete_user(db, NAME, PASSWORD).await;
    let _ = data.delete_user(db, NAME2, PASSWORD).await;
    let _ = data.delete_user(db, NAME3, PASSWORD).await;

    //// update username of nonexistent user
    //data.update_username(NAME, PASSWORD).await.err();
    assert_eq!(
        data.update_username(db, NAME, PASSWORD).await.err(),
        Some(ServiceError::AccountNotFound)
    );

    // update secret of nonexistent user
    assert_eq!(
        data.get_secret(db, NAME).await.err(),
        Some(ServiceError::AccountNotFound)
    );

    // get secret of non-existent account
    assert_eq!(
        data.update_user_secret(db, NAME).await.err(),
        Some(ServiceError::AccountNotFound)
    );

    //update email of nonexistent user
    assert_eq!(
        data.set_email(db, NAME, EMAIL).await.err(),
        Some(ServiceError::AccountNotFound)
    );

    // check username exists for non existent account
    assert!(!data.username_exists(db, NAME).await.unwrap().exists);
    // check username email for non existent account
    assert!(!data.email_exists(db, EMAIL).await.unwrap().exists);

    let mut register_payload = Register {
        username: NAME.into(),
        password: PASSWORD.into(),
        confirm_password: PASSWORD.into(),
        email: Some(EMAIL.into()),
    };
    data.register(db, &register_payload).await.unwrap();
    register_payload.username = NAME2.into();
    register_payload.email = Some(EMAIL2.into());
    data.register(db, &register_payload).await.unwrap();

    // check username exists
    assert!(data.username_exists(db, NAME).await.unwrap().exists);
    assert!(data.username_exists(db, NAME2).await.unwrap().exists);
    // check email exists
    assert!(data.email_exists(db, EMAIL).await.unwrap().exists);

    // check if get user secret works
    let secret = data.get_secret(db, NAME).await.unwrap();

    data.update_user_secret(db, NAME).await.unwrap();
    let new_secret = data.get_secret(db, NAME).await.unwrap();
    assert_ne!(secret.secret, new_secret.secret);

    // update username
    data.update_username(db, NAME2, NAME3).await.unwrap();
    assert!(!data.username_exists(db, NAME2).await.unwrap().exists);
    assert!(data.username_exists(db, NAME3).await.unwrap().exists);

    assert!(matches!(
        data.update_username(db, NAME3, NAME).await.err(),
        Some(ServiceError::UsernameTaken)
    ));

    // update email
    assert_eq!(
        data.set_email(db, NAME, EMAIL2).await.err(),
        Some(ServiceError::EmailTaken)
    );
    data.set_email(db, NAME, EMAIL3).await.unwrap();

    // change password
    let mut change_password_req = ChangePasswordReqest {
        password: PASSWORD.into(),
        new_password: NAME.into(),
        confirm_new_password: PASSWORD.into(),
    };
    assert_eq!(
        data.change_password(db, NAME, &change_password_req)
            .await
            .err(),
        Some(ServiceError::PasswordsDontMatch)
    );

    change_password_req.confirm_new_password = NAME.into();
    data.change_password(db, NAME, &change_password_req)
        .await
        .unwrap();
}

#[actix_rt::test]
async fn email_udpate_password_validation_del_userworks(data: Arc<Data>, db: DB) {
    const NAME: &str = "testuser2";
    const PASSWORD: &str = "longpassword2";
    const EMAIL: &str = "testuser1@a.com2";
    const NAME2: &str = "eupdauser";
    const EMAIL2: &str = "eupdauser@a.com";
    let db = &db;

        data.delete_user(db, NAME, PASSWORD).await;
        data.delete_user(db, NAME2,PASSWORD).await;

    let _ = data.register_and_signin(db, NAME2, EMAIL2, PASSWORD).await;
    let (data, _creds, signin_resp) = data.register_and_signin(db, NAME, EMAIL, PASSWORD).await;
    let cookies = get_cookie!(signin_resp);
    let app = get_app!(data).await;

    // update email
    let mut email_payload = Email {
        email: EMAIL.into(),
    };
    let email_update_resp = test::call_service(
        &app,
        post_request!(&email_payload, ROUTES.account.update_email)
            //post_request!(&email_payload, EMAIL_UPDATE)
            .cookie(cookies.clone())
            .to_request(),
    )
    .await;
    assert_eq!(email_update_resp.status(), StatusCode::OK);

    // check duplicate email while duplicate email
    email_payload.email = EMAIL2.into();
    data.bad_post_req_test(
        db,
        NAME,
        PASSWORD,
        ROUTES.account.update_email,
        &email_payload,
        ServiceError::EmailTaken,
    )
    .await;

    // wrong password while deleting account
    let mut payload = Password {
        password: NAME.into(),
    };
    data.bad_post_req_test(
        db,
        NAME,
        PASSWORD,
        ROUTES.account.delete,
        &payload,
        ServiceError::WrongPassword,
    )
    .await;

    // delete account
    payload.password = PASSWORD.into();
    let delete_user_resp = test::call_service(
        &app,
        post_request!(&payload, ROUTES.account.delete)
            .cookie(cookies.clone())
            .to_request(),
    )
    .await;

    assert_eq!(delete_user_resp.status(), StatusCode::OK);

    // try to delete an account that doesn't exist
    let account_not_found_resp = test::call_service(
        &app,
        post_request!(&payload, ROUTES.account.delete)
            .cookie(cookies)
            .to_request(),
    )
    .await;
    assert_eq!(account_not_found_resp.status(), StatusCode::NOT_FOUND);
    let txt: ErrorToResponse = test::read_body_json(account_not_found_resp).await;
    assert_eq!(txt.error, format!("{}", ServiceError::AccountNotFound));
}

async fn username_update_works(data: Arc<Data>, db: DB) {
    const NAME: &str = "testuserupda";
    const EMAIL: &str = "testuserupda@sss.com";
    const EMAIL2: &str = "testuserupda2@sss.com";
    const PASSWORD: &str = "longpassword2";
    const NAME2: &str = "terstusrtds";
    const NAME_CHANGE: &str = "terstusrtdsxx";

    let db = &db;

    futures::join!(
        data.delete_user(db, NAME, PASSWORD),
        data.delete_user(db, NAME2, PASSWORD),
        data.delete_user(db, NAME_CHANGE, PASSWORD)
    );

    let _ = data.register_and_signin(db, NAME2, EMAIL2, PASSWORD).await;
    let (_creds, signin_resp) = data.register_and_signin(db, NAME, EMAIL, PASSWORD).await;
    let cookies = get_cookie!(signin_resp);
    let app = get_app!(data).await;

    // update username
    let mut username_udpate = Username {
        username: NAME_CHANGE.into(),
    };
    let username_update_resp = test::call_service(
        &app,
        post_request!(&username_udpate, ROUTES.account.update_username)
            .cookie(cookies)
            .to_request(),
    )
    .await;
    assert_eq!(username_update_resp.status(), StatusCode::OK);

    // check duplicate username with duplicate username
    username_udpate.username = NAME2.into();
    data.bad_post_req_test(
    db,
        NAME_CHANGE,
        PASSWORD,
        ROUTES.account.update_username,
        &username_udpate,
        ServiceError::UsernameTaken,
    )
    .await;
}
