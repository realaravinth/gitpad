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
use actix_web::http::StatusCode;
use actix_web::test;

use super::*;

use crate::data::Data;
use crate::tests::*;
use crate::*;

#[actix_rt::test]
async fn postgres_gists_work() {
    let (db, data) = sqlx_postgres::get_data().await;
    gists_new_route_works(data.clone(), db.clone()).await;
}

#[actix_rt::test]
async fn sqlite_gists_work() {
    let (db, data) = sqlx_sqlite::get_data().await;
    gists_new_route_works(data.clone(), db.clone()).await;
}

async fn gists_new_route_works(data: Arc<Data>, db: BoxDB) {
    const NAME: &str = "newgisttestuserexists";
    const PASSWORD: &str = "longpassword2";
    const EMAIL: &str = "newgisttestuserexists@a.com2";
    let db = &db;

    let _ = data.delete_user(db, NAME, PASSWORD).await;

    let (_, signin_resp) = data.register_and_signin(db, NAME, EMAIL, PASSWORD).await;
    let cookies = get_cookie!(signin_resp);
    let app = get_app!(data, db).await;
    let new_gist = get_request!(&app, PAGES.gist.new, cookies.clone());
    assert_eq!(new_gist.status(), StatusCode::OK);
}
