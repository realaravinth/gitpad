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
use actix_http::header;
use actix_web::http::StatusCode;
use actix_web::test;
use actix_web::ResponseError;

use db_core::prelude::*;
use pages::routes::PostCommentPath;

use super::new::*;

use crate::api::v1::gists::PostCommentRequest;
use crate::data::Data;
use crate::errors::*;
use crate::tests::*;
use crate::*;

#[actix_rt::test]
async fn postgres_pages_gists_work() {
    let (db, data) = sqlx_postgres::get_data().await;
    gists_new_route_works(data.clone(), db.clone()).await;
}

#[actix_rt::test]
async fn sqlite_pages_gists_work() {
    let (db, data) = sqlx_sqlite::get_data().await;
    gists_new_route_works(data.clone(), db.clone()).await;
}

async fn gists_new_route_works(data: Arc<Data>, db: BoxDB) {
    const NAME: &str = "newgisttestuserexists";
    const PASSWORD: &str = "longpassword2";
    const EMAIL: &str = "newgisttestuserexists@a.com2";
    const COMMENT: &str = "this string is never used anywhere but for commenting, so that I can get away with body inlcudes";
    let db = &db;

    let _ = data.delete_user(db, NAME, PASSWORD).await;

    let (_, signin_resp) = data.register_and_signin(db, NAME, EMAIL, PASSWORD).await;
    let cookies = get_cookie!(signin_resp);
    let app = get_app!(data, db).await;
    let new_gist = get_request!(&app, PAGES.gist.new, cookies.clone());
    assert_eq!(new_gist.status(), StatusCode::OK);
    let files = FieldNames::<String>::new(1);

    // create gist
    let payload = serde_json::json!({
        "description": "",
        "visibility": GistVisibility::Private.to_str(),
        files.filename.clone() : "foo.md",
        files.content.clone() : "foo.md",
    });

    let resp = test::call_service(
        &app,
        post_request!(&payload, PAGES.gist.new, FORM)
            .cookie(cookies.clone())
            .to_request(),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::FOUND);
    let gist_id = resp.headers().get(header::LOCATION).unwrap();
    let resp = get_request!(&app, gist_id.to_str().unwrap(), cookies.clone());
    assert_eq!(resp.status(), StatusCode::OK);

    // add new file during gist creation
    let payload = serde_json::json!({
        "description": "",
        "visibility": GistVisibility::Private.to_str(),
        files.filename.clone() : "foo.md",
        files.content.clone() : "foo.md",
        "add_file": "",
    });

    let resp = test::call_service(
        &app,
        post_request!(&payload, PAGES.gist.new, FORM)
            .cookie(cookies.clone())
            .to_request(),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let empty_gist = test::call_service(
        &app,
        post_request!(&serde_json::Value::default(), PAGES.gist.new, FORM)
            .cookie(cookies.clone())
            .to_request(),
    )
    .await;
    assert_eq!(empty_gist.status(), ServiceError::GistEmpty.status_code());

    // get gist

    let mut route_iter = gist_id.to_str().unwrap().split('/');
    let name = route_iter.nth(1).unwrap();
    let gist = route_iter.next().unwrap();
    let gist_route_componenet = PostCommentPath {
        username: name.to_string(),
        gist: gist.to_string(),
    };
    let gist_html_route = PAGES.gist.get_gist_route(&gist_route_componenet);
    let gist_html_page = get_request!(&app, &gist_html_route);
    assert_eq!(gist_html_page.status(), StatusCode::OK);

    // post comment
    let comment_url = PAGES.gist.get_post_comment_route(&gist_route_componenet);
    let comment = PostCommentRequest {
        comment: COMMENT.into(),
    };
    let comment_resp = test::call_service(
        &app,
        post_request!(&comment, &comment_url, FORM)
            .cookie(cookies)
            .to_request(),
    )
    .await;
    assert_eq!(comment_resp.status(), StatusCode::FOUND);
}
