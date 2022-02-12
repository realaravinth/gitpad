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

use actix_identity::Identity;
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use crate::data::api::v1::account::*;
use crate::data::api::v1::auth::Password;
use crate::errors::*;
use crate::AppData;

#[cfg(test)]
pub mod test;

pub use super::auth;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AccountCheckPayload {
    pub val: String,
}

pub fn services(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(username_exists);
    cfg.service(set_username);
    cfg.service(email_exists);
    cfg.service(set_email);
    cfg.service(delete_account);
    cfg.service(update_user_password);
    cfg.service(get_secret);
    cfg.service(update_user_secret);
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Email {
    pub email: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Username {
    pub username: String,
}

/// update username
#[my_codegen::post(
    path = "crate::V1_API_ROUTES.account.update_username",
    wrap = "super::get_auth_middleware()"
)]
async fn set_username(
    id: Identity,
    payload: web::Json<Username>,
    data: AppData,
    db: crate::DB,
) -> ServiceResult<impl Responder> {
    let username = id.identity().unwrap();

    let new_name = data
        .update_username(&(**db), &username, &payload.username)
        .await?;

    id.forget();
    id.remember(new_name);

    Ok(HttpResponse::Ok())
}

#[my_codegen::post(path = "crate::V1_API_ROUTES.account.username_exists")]
async fn username_exists(
    payload: web::Json<AccountCheckPayload>,
    data: AppData,
    db: crate::DB,
) -> ServiceResult<impl Responder> {
    Ok(HttpResponse::Ok().json(data.username_exists(&(**db), &payload.val).await?))
}

#[my_codegen::post(path = "crate::V1_API_ROUTES.account.email_exists")]
pub async fn email_exists(
    payload: web::Json<AccountCheckPayload>,
    data: AppData,
    db: crate::DB,
) -> ServiceResult<impl Responder> {
    Ok(HttpResponse::Ok().json(data.email_exists(&(**db), &payload.val).await?))
}

/// update email
#[my_codegen::post(
    path = "crate::V1_API_ROUTES.account.update_email",
    wrap = "super::get_auth_middleware()"
)]
async fn set_email(
    id: Identity,
    payload: web::Json<Email>,
    data: AppData,
    db: crate::DB,
) -> ServiceResult<impl Responder> {
    let username = id.identity().unwrap();
    data.set_email(&(**db), &username, &payload.email).await?;
    Ok(HttpResponse::Ok())
}

#[my_codegen::post(
    path = "crate::V1_API_ROUTES.account.delete",
    wrap = "super::get_auth_middleware()"
)]
async fn delete_account(
    id: Identity,
    payload: web::Json<Password>,
    data: AppData,
    db: crate::DB,
) -> ServiceResult<impl Responder> {
    let username = id.identity().unwrap();

    data.delete_user(&(**db), &username, &payload.password)
        .await?;
    id.forget();
    Ok(HttpResponse::Ok())
}

#[my_codegen::post(
    path = "crate::V1_API_ROUTES.account.update_password",
    wrap = "super::get_auth_middleware()"
)]
async fn update_user_password(
    id: Identity,
    data: AppData,
    db: crate::DB,
    payload: web::Json<ChangePasswordReqest>,
) -> ServiceResult<impl Responder> {
    let username = id.identity().unwrap();
    let payload = payload.into_inner();
    data.change_password(&(**db), &username, &payload).await?;

    Ok(HttpResponse::Ok())
}

#[my_codegen::get(
    path = "crate::V1_API_ROUTES.account.get_secret",
    wrap = "super::get_auth_middleware()"
)]
async fn get_secret(id: Identity, data: AppData, db: crate::DB) -> ServiceResult<impl Responder> {
    let username = id.identity().unwrap();
    let secret = data.get_secret(&(**db), &username).await?;
    Ok(HttpResponse::Ok().json(secret))
}

#[my_codegen::post(
    path = "crate::V1_API_ROUTES.account.update_secret",
    wrap = "super::get_auth_middleware()"
)]
async fn update_user_secret(
    id: Identity,
    data: AppData,
    db: crate::DB,
) -> ServiceResult<impl Responder> {
    let username = id.identity().unwrap();
    let _secret = data.update_user_secret(&(**db), &username).await?;

    Ok(HttpResponse::Ok())
}
