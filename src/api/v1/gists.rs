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
use actix_identity::Identity;
use actix_web::*;

use db_core::prelude::*;
use serde::{Deserialize, Serialize};

use super::routes::{GetFilePath, PostCommentPath};
use crate::data::api::v1::gists::{CreateGist, FileInfo, GistID};
use crate::errors::*;
use crate::utils::escape_spaces;
use crate::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGistRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub visibility: GistVisibility,
    pub files: Vec<FileInfo>,
}

impl CreateGistRequest {
    pub fn to_create_gist<'a>(&'a self, owner: &'a str) -> CreateGist<'a> {
        CreateGist {
            owner,
            description: self.description.as_deref(),
            visibility: &self.visibility,
        }
    }
}

pub fn services(cfg: &mut web::ServiceConfig) {
    cfg.service(new);
    cfg.service(get_file);
    cfg.service(post_comment);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGistResp {
    /// public ID
    pub id: String,
}

#[my_codegen::post(
    path = "crate::V1_API_ROUTES.gist.new",
    wrap = "super::get_auth_middleware()"
)]
async fn new(
    payload: web::Json<CreateGistRequest>,
    data: AppData,
    id: Identity,
    db: crate::DB,
) -> ServiceResult<impl Responder> {
    let username = id.identity().unwrap();
    let mut gist = data
        .new_gist(db.as_ref(), &payload.to_create_gist(&username))
        .await?;
    data.write_file(
        db.as_ref(),
        GistID::Repository(&mut gist.repository),
        &payload.files,
    )
    .await?;
    let resp = CreateGistResp { id: gist.id };
    Ok(HttpResponse::Ok().json(&resp))
}

#[my_codegen::get(path = "crate::V1_API_ROUTES.gist.get_file")]
async fn get_file(
    path: web::Path<GetFilePath>,
    data: AppData,
    id: Identity,
    db: crate::DB,
) -> ServiceResult<impl Responder> {
    let gist = db.get_gist(&path.gist).await?;
    match gist.visibility {
        GistVisibility::Public | GistVisibility::Unlisted => {
            let contents = data
                .read_file(
                    db.as_ref(),
                    GistID::ID(&gist.public_id),
                    &escape_spaces(&path.file),
                )
                .await?;
            Ok(HttpResponse::Ok().json(contents))
        }
        GistVisibility::Private => {
            if let Some(username) = id.identity() {
                if gist.owner == username {
                    let contents = data
                        .read_file(
                            db.as_ref(),
                            GistID::ID(&gist.public_id),
                            &escape_spaces(&path.file),
                        )
                        .await?;
                    return Ok(HttpResponse::Ok().json(contents));
                }
            };
            Err(ServiceError::GistNotFound)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostCommentRequest {
    pub comment: String,
}

#[my_codegen::post(
    path = "crate::V1_API_ROUTES.gist.post_comment",
    wrap = "super::get_auth_middleware()"
)]
async fn post_comment(
    payload: web::Json<PostCommentRequest>,
    path: web::Path<PostCommentPath>,
    id: Identity,
    db: crate::DB,
) -> ServiceResult<impl Responder> {
    let comment = payload.comment.trim();
    if comment.is_empty() {
        return Err(ServiceError::EmptyComment);
    }

    let username = id.identity().unwrap();
    let gist = db.get_gist(&path.gist).await?;
    if gist.visibility == GistVisibility::Private && username != gist.owner {
        return Err(ServiceError::GistNotFound);
    }

    let msg = CreateGistComment {
        owner: &username,
        gist_public_id: &path.gist,
        comment: &payload.comment,
    };

    db.new_comment(&msg).await?; //  TODO get comment ID
    Ok(HttpResponse::Ok().finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::api::v1::gists::{ContentType, FileType};
    use crate::tests::*;

    use crate::utils::escape_spaces;
    #[actix_rt::test]
    async fn test_new_gist_works_postgres() {
        let (db, data) = sqlx_postgres::get_data().await;
        new_gist_test_runner(&data, &db).await;
    }
    #[actix_rt::test]
    async fn test_new_gist_works_sqlite() {
        let (db, data) = sqlx_sqlite::get_data().await;
        new_gist_test_runner(&data, &db).await;
    }

    async fn new_gist_test_runner(data: &Arc<Data>, db: &BoxDB) {
        const NAME: &str = "httpgisttestuser";
        const NAME2: &str = "httpgisttestuser2";
        const EMAIL: &str = "httpgisttestuser@sss.com";
        const EMAIL2: &str = "httpgisttestuse2r@sss.com";
        const PASSWORD: &str = "longpassword2";

        let _ = futures::join!(
            data.delete_user(db, NAME, PASSWORD),
            data.delete_user(db, NAME2, PASSWORD),
        );

        let (_creds, signin_resp2) = data.register_and_signin(db, NAME2, EMAIL2, PASSWORD).await;
        let cookies2 = get_cookie!(signin_resp2);
        let (_creds, signin_resp) = data.register_and_signin(db, NAME, EMAIL, PASSWORD).await;
        let cookies = get_cookie!(signin_resp);
        let app = get_app!(data, db).await;

        let files = [
            FileInfo {
                filename: "foo".into(),
                content: FileType::File(ContentType::Text("foobar".into())),
            },
            FileInfo {
                filename: "bar".into(),
                content: FileType::File(ContentType::Text("foobar".into())),
            },
            FileInfo {
                filename: "foo bar".into(),
                content: FileType::File(ContentType::Text("foobar".into())),
            },
        ];

        let create_gist_msg = CreateGistRequest {
            description: None,
            visibility: GistVisibility::Public,
            files: files.to_vec(),
        };

        let create_gist_resp = test::call_service(
            &app,
            post_request!(&create_gist_msg, V1_API_ROUTES.gist.new)
                .cookie(cookies.clone())
                .to_request(),
        )
        .await;
        assert_eq!(create_gist_resp.status(), StatusCode::OK);
        let gist_id: CreateGistResp = test::read_body_json(create_gist_resp).await;
        let gist_id = gist_id.id;

        data.gist_created_test_helper(db, &gist_id, NAME).await;
        data.gist_files_written_helper(db, &gist_id, &files).await;

        // get gists
        // 1. Public gists
        let mut get_file_path = GetFilePath {
            username: NAME.into(),
            gist: gist_id.clone(),
            file: "".into(),
        };
        for file in files.iter() {
            // with owner cookies
            get_file_path.file = file.filename.clone();
            let path = V1_API_ROUTES.gist.get_file_route(&get_file_path);
            println!("Trying to get file {path}");
            let resp = get_request!(&app, &path, cookies.clone());
            assert_eq!(resp.status(), StatusCode::OK);
            let content: FileInfo = test::read_body_json(resp).await;

            let req_escaped_file = FileInfo {
                filename: escape_spaces(&file.filename),
                content: file.content.clone(),
            };
            assert_eq!(&content, &req_escaped_file);

            // unauthenticated user
            let resp = get_request!(&app, &path);
            assert_eq!(resp.status(), StatusCode::OK);
            let content: FileInfo = test::read_body_json(resp).await;
            let req_escaped_file = FileInfo {
                filename: escape_spaces(&file.filename),
                content: file.content.clone(),
            };
            assert_eq!(&content, &req_escaped_file);

            // non-owner user
            let resp = get_request!(&app, &path, cookies2.clone());
            assert_eq!(resp.status(), StatusCode::OK);
            let content: FileInfo = test::read_body_json(resp).await;
            let req_escaped_file = FileInfo {
                filename: escape_spaces(&file.filename),
                content: file.content.clone(),
            };
            assert_eq!(&content, &req_escaped_file);
        }
        // 2. Unlisted gists
        let one_file = [files[0].clone()];
        let mut msg = CreateGistRequest {
            description: None,
            visibility: GistVisibility::Unlisted,
            files: one_file.to_vec(),
        };

        let create_gist_resp = test::call_service(
            &app,
            post_request!(&msg, V1_API_ROUTES.gist.new)
                .cookie(cookies.clone())
                .to_request(),
        )
        .await;
        assert_eq!(create_gist_resp.status(), StatusCode::OK);
        let unlisted: CreateGistResp = test::read_body_json(create_gist_resp).await;
        let unlisted = unlisted.id;

        get_file_path.gist = unlisted.clone();
        for file in one_file.iter() {
            // requesting user is owner
            get_file_path.file = file.filename.clone();
            let path = V1_API_ROUTES.gist.get_file_route(&get_file_path);
            println!("Trying to get file {path}");
            let resp = get_request!(&app, &path, cookies.clone());
            assert_eq!(resp.status(), StatusCode::OK);
            let content: FileInfo = test::read_body_json(resp).await;
            let req_escaped_file = FileInfo {
                filename: escape_spaces(&file.filename),
                content: file.content.clone(),
            };
            assert_eq!(&content, &req_escaped_file);

            // unauthenticated
            let resp = get_request!(&app, &path);
            assert_eq!(resp.status(), StatusCode::OK);
            let content: FileInfo = test::read_body_json(resp).await;
            let req_escaped_file = FileInfo {
                filename: escape_spaces(&file.filename),
                content: file.content.clone(),
            };
            assert_eq!(&content, &req_escaped_file);

            // requesting user is not owner
            let resp = get_request!(&app, &path, cookies2.clone());
            assert_eq!(resp.status(), StatusCode::OK);
            let content: FileInfo = test::read_body_json(resp).await;
            let req_escaped_file = FileInfo {
                filename: escape_spaces(&file.filename),
                content: file.content.clone(),
            };
            assert_eq!(&content, &req_escaped_file);
        }

        // 2. Private gists
        msg.visibility = GistVisibility::Private;

        let create_gist_resp = test::call_service(
            &app,
            post_request!(&msg, V1_API_ROUTES.gist.new)
                .cookie(cookies.clone())
                .to_request(),
        )
        .await;
        assert_eq!(create_gist_resp.status(), StatusCode::OK);
        let private: CreateGistResp = test::read_body_json(create_gist_resp).await;
        let private = private.id;

        get_file_path.gist = private.clone();
        for file in one_file.iter() {
            get_file_path.file = file.filename.clone();
            let path = V1_API_ROUTES.gist.get_file_route(&get_file_path);
            println!("Trying to get file {path}");
            // requesting user is owner
            let resp = get_request!(&app, &path, cookies.clone());
            assert_eq!(resp.status(), StatusCode::OK);
            let content: FileInfo = test::read_body_json(resp).await;

            let req_escaped_file = FileInfo {
                filename: escape_spaces(&file.filename),
                content: file.content.clone(),
            };
            assert_eq!(&content, &req_escaped_file);

            // requesting user is unauthenticated
            let resp = get_request!(&app, &path);
            assert_eq!(resp.status(), StatusCode::NOT_FOUND);
            let txt: ErrorToResponse = test::read_body_json(resp).await;
            assert_eq!(txt.error, format!("{}", ServiceError::GistNotFound));

            // requesting user is not owner
            let resp = get_request!(&app, &path, cookies2.clone());
            assert_eq!(resp.status(), StatusCode::NOT_FOUND);
            let txt: ErrorToResponse = test::read_body_json(resp).await;
            assert_eq!(txt.error, format!("{}", ServiceError::GistNotFound));
        }

        println!("testing comments");
        let mut create_comment = PostCommentPath {
            username: NAME2.into(),
            gist: gist_id.clone(),
        };

        let mut comment = PostCommentRequest { comment: "".into() };
        println!("empty comment");
        // empty comment
        data.bad_post_req_test(
            db,
            NAME,
            PASSWORD,
            V1_API_ROUTES.gist.post_comment,
            &comment,
            ServiceError::EmptyComment,
        )
        .await;

        comment.comment = "foo".into();

        create_comment.gist = "gistnotexist".into();
        let post_comment_path = V1_API_ROUTES.gist.get_post_comment_route(&create_comment);
        println!("gist not found");
        data.bad_post_req_test(
            db,
            NAME,
            PASSWORD,
            &post_comment_path,
            &comment,
            ServiceError::GistNotFound,
        )
        .await;

        println!("comment OK");
        create_comment.gist = gist_id;
        let post_comment_path = V1_API_ROUTES.gist.get_post_comment_route(&create_comment);
        let resp = test::call_service(
            &app,
            post_request!(&comment, &post_comment_path)
                .cookie(cookies.clone())
                .to_request(),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);

        println!("comment OK");
        create_comment.gist = unlisted;
        let post_comment_path = V1_API_ROUTES.gist.get_post_comment_route(&create_comment);
        let resp = test::call_service(
            &app,
            post_request!(&comment, &post_comment_path)
                .cookie(cookies.clone())
                .to_request(),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);

        println!("comment OK");
        create_comment.gist = private.clone();
        let post_comment_path = V1_API_ROUTES.gist.get_post_comment_route(&create_comment);
        let resp = test::call_service(
            &app,
            post_request!(&comment, &post_comment_path)
                .cookie(cookies.clone())
                .to_request(),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);

        // commenting on private gist
        println!("private gist, not OK");
        create_comment.gist = private.clone();
        let post_comment_path = V1_API_ROUTES.gist.get_post_comment_route(&create_comment);
        data.bad_post_req_test(
            db,
            NAME2,
            PASSWORD,
            &post_comment_path,
            &comment,
            ServiceError::GistNotFound,
        )
        .await;
    }
}
