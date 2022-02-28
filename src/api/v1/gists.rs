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

use super::routes::{GetCommentPath, GetFilePath, PostCommentPath};
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
    cfg.service(get_comment);
    cfg.service(get_gist_comments);
    cfg.service(delete_comment);
    cfg.service(index);
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
    if payload.files.is_empty() {
        return Err(ServiceError::GistEmpty);
    }

    let username = id.identity().unwrap();
    let mut gist = data
        .new_gist(db.as_ref(), &payload.to_create_gist(&username))
        .await?;
    data.write_file(
        db.as_ref(),
        &mut GistID::Repository(&mut gist.repository),
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
                    &GistID::ID(&gist.public_id),
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
                            &GistID::ID(&gist.public_id),
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostCommentResp {
    pub id: i64,
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

    let resp = PostCommentResp {
        id: db.new_comment(&msg).await?,
    };
    Ok(HttpResponse::Ok().json(resp))
}

#[my_codegen::get(path = "crate::V1_API_ROUTES.gist.get_comment")]
async fn get_comment(
    path: web::Path<GetCommentPath>,
    id: Identity,
    db: crate::DB,
) -> ServiceResult<impl Responder> {
    let gist = db.get_gist(&path.gist).await?;

    match gist.visibility {
        GistVisibility::Public | GistVisibility::Unlisted => {
            let comment = db.get_comment_by_id(path.comment_id).await?;
            Ok(HttpResponse::Ok().json(comment))
        }
        GistVisibility::Private => {
            if let Some(username) = id.identity() {
                if gist.owner == username {
                    let comment = db.get_comment_by_id(path.comment_id).await?;
                    return Ok(HttpResponse::Ok().json(comment));
                }
            };
            Err(ServiceError::GistNotFound)
        }
    }
}

#[my_codegen::get(path = "crate::V1_API_ROUTES.gist.get_gist_comments")]
async fn get_gist_comments(
    path: web::Path<PostCommentPath>,
    id: Identity,
    db: crate::DB,
) -> ServiceResult<impl Responder> {
    let gist = db.get_gist(&path.gist).await?;

    match gist.visibility {
        GistVisibility::Public | GistVisibility::Unlisted => {
            let comments = db.get_comments_on_gist(&path.gist).await?;
            Ok(HttpResponse::Ok().json(comments))
        }
        GistVisibility::Private => {
            if let Some(username) = id.identity() {
                if gist.owner == username {
                    let comments = db.get_comments_on_gist(&path.gist).await?;
                    return Ok(HttpResponse::Ok().json(comments));
                }
            };
            Err(ServiceError::GistNotFound)
        }
    }
}

#[my_codegen::delete(
    path = "crate::V1_API_ROUTES.gist.delete_comment",
    wrap = "super::get_auth_middleware()"
)]
async fn delete_comment(
    path: web::Path<GetCommentPath>,
    id: Identity,
    db: crate::DB,
) -> ServiceResult<impl Responder> {
    let gist = db.get_gist(&path.gist).await?;
    let comment = db.get_comment_by_id(path.comment_id).await?;
    let username = id.identity().unwrap();
    if username != comment.owner {
        match gist.visibility {
            GistVisibility::Public | GistVisibility::Unlisted => {
                Err(ServiceError::UnauthorizedOperation(
                    "This user is not the owner of the comment to delete it".into(),
                ))
            }
            GistVisibility::Private => Err(ServiceError::GistNotFound),
        }
    } else {
        db.delete_comment(&username, comment.id).await?;
        Ok(HttpResponse::Ok())
    }
}

#[my_codegen::get(
    path = "crate::V1_API_ROUTES.gist.gist_index",
    wrap = "super::get_auth_middleware()"
)]
async fn index(
    path: web::Path<PostCommentPath>,
    id: Identity,
    db: crate::DB,
    data: AppData,
) -> ServiceResult<impl Responder> {
    let username = id.identity().unwrap();
    let gist = db.get_gist(&path.gist).await?;
    if gist.visibility == GistVisibility::Private && username != gist.owner {
        return Err(ServiceError::GistNotFound);
    }

    let resp = data
        .gist_preview(db.as_ref(), &mut GistID::ID(&path.gist))
        .await?;

    Ok(HttpResponse::Ok().json(resp))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::api::v1::gists::{ContentType, FileType, GistInfo};
    use crate::tests::*;
    use actix_web::ResponseError;

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
        let mut msg = CreateGistRequest {
            description: None,
            visibility: GistVisibility::Unlisted,
            files: files.to_vec(),
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
        for file in files.iter() {
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
        for file in files.iter() {
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

        /* +++++++++++++++++++++++++++++++
         *              COMMENTS
         * +++++++++++++++++++++++++++++++
         */

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

        let mut comment_ids = Vec::with_capacity(3);

        println!("comment OK");
        create_comment.gist = gist_id.clone();
        let post_comment_path = V1_API_ROUTES.gist.get_post_comment_route(&create_comment);
        let resp = test::call_service(
            &app,
            post_request!(&comment, &post_comment_path)
                .cookie(cookies.clone())
                .to_request(),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
        let comment_resp: PostCommentResp = test::read_body_json(resp).await;
        comment_ids.push((
            comment_resp,
            comment.clone(),
            gist_id.clone(),
            GistVisibility::Public,
        ));

        println!("comment OK");
        create_comment.gist = unlisted.clone();
        let post_comment_path = V1_API_ROUTES.gist.get_post_comment_route(&create_comment);
        let resp = test::call_service(
            &app,
            post_request!(&comment, &post_comment_path)
                .cookie(cookies.clone())
                .to_request(),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
        let comment_resp: PostCommentResp = test::read_body_json(resp).await;
        comment_ids.push((
            comment_resp,
            comment.clone(),
            unlisted.clone(),
            GistVisibility::Unlisted,
        ));

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
        let comment_resp: PostCommentResp = test::read_body_json(resp).await;
        comment_ids.push((
            comment_resp,
            comment.clone(),
            private.clone(),
            GistVisibility::Private,
        ));

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

        /*
         * +++++++++++++++++++++++++++++++++++
         *          GET COMMENT
         * +++++++++++++++++++++++++++++++++++
         */

        // gist not found
        let mut get_comment_path_component = GetCommentPath {
            gist: "gistdoesntexist".into(),
            username: NAME.into(),
            comment_id: 466767,
        };
        let get_comment_path = V1_API_ROUTES
            .gist
            .get_get_comment_route(&get_comment_path_component);
        println!("getting comment; gist doesn't exist");
        let resp = get_request!(&app, &get_comment_path);
        assert_eq!(resp.status(), ServiceError::GistNotFound.status_code());
        let resp_err: ErrorToResponse = test::read_body_json(resp).await;
        assert_eq!(resp_err.error, format!("{}", ServiceError::GistNotFound));

        // private gist
        get_comment_path_component.gist = private.clone();
        let get_comment_path = V1_API_ROUTES
            .gist
            .get_get_comment_route(&get_comment_path_component);
        println!("getting comment; private gist");
        let resp = get_request!(&app, &get_comment_path);
        assert_eq!(resp.status(), ServiceError::GistNotFound.status_code());
        let resp_err: ErrorToResponse = test::read_body_json(resp).await;
        assert_eq!(resp_err.error, format!("{}", ServiceError::GistNotFound));

        // comment not found
        get_comment_path_component.gist = gist_id.clone();
        let get_comment_path = V1_API_ROUTES
            .gist
            .get_get_comment_route(&get_comment_path_component);
        println!("getting comment; comment doesn't exist");
        let resp = get_request!(&app, &get_comment_path);
        assert_eq!(resp.status(), ServiceError::CommentNotFound.status_code());
        let resp_err: ErrorToResponse = test::read_body_json(resp).await;
        assert_eq!(resp_err.error, format!("{}", ServiceError::CommentNotFound));

        for (comment_id, comment_payload, gist, visibility) in comment_ids.iter() {
            let component = GetCommentPath {
                gist: gist.into(),
                username: NAME.into(),
                comment_id: comment_id.id,
            };

            if visibility == &GistVisibility::Private {
                println!("getting comment; private gist but user==owner");
                let path = V1_API_ROUTES.gist.get_get_comment_route(&component);
                let resp = get_request!(&app, &path, cookies.clone());
                assert_eq!(resp.status(), StatusCode::OK);
                let comment: GistComment = test::read_body_json(resp).await;
                assert_eq!(comment.comment, comment_payload.comment);

                println!("getting comment; private gist but user is unauthenticated");
                let resp = get_request!(&app, &path);
                assert_eq!(resp.status(), ServiceError::GistNotFound.status_code());
                let resp_err: ErrorToResponse = test::read_body_json(resp).await;
                assert_eq!(resp_err.error, format!("{}", ServiceError::GistNotFound));

                println!("getting comment; private gist but user != owner");
                let resp = get_request!(&app, &path, cookies2.clone());
                assert_eq!(resp.status(), ServiceError::GistNotFound.status_code());
                let err: ErrorToResponse = test::read_body_json(resp).await;
                assert_eq!(err.error, format!("{}", ServiceError::GistNotFound));
            } else {
                let path = V1_API_ROUTES.gist.get_get_comment_route(&component);
                let resp = get_request!(&app, &path, cookies.clone());
                assert_eq!(resp.status(), StatusCode::OK);
                let comment: GistComment = test::read_body_json(resp).await;
                assert_eq!(comment.comment, comment_payload.comment);
            }
        }

        /*
         * +++++++++++++++++++++++++++++++++++
         *          GET GIST COMMENT
         * +++++++++++++++++++++++++++++++++++
         */

        // gist not found
        let mut get_gist_comments_component = PostCommentPath {
            gist: "gistdoesntexist".into(),
            username: NAME.into(),
        };
        let get_comment_path = V1_API_ROUTES
            .gist
            .get_gist_comments(&get_gist_comments_component);
        println!("getting comments; gist doesn't exist");
        let resp = get_request!(&app, &get_comment_path);
        assert_eq!(resp.status(), ServiceError::GistNotFound.status_code());
        let resp_err: ErrorToResponse = test::read_body_json(resp).await;
        assert_eq!(resp_err.error, format!("{}", ServiceError::GistNotFound));

        // private gist
        get_gist_comments_component.gist = private.clone();
        let get_comment_path = V1_API_ROUTES
            .gist
            .get_gist_comments(&get_gist_comments_component);
        println!("getting comments; private gist");
        let resp = get_request!(&app, &get_comment_path);
        assert_eq!(resp.status(), ServiceError::GistNotFound.status_code());
        let resp_err: ErrorToResponse = test::read_body_json(resp).await;
        assert_eq!(resp_err.error, format!("{}", ServiceError::GistNotFound));

        for (_comment_id, comment_payload, gist, visibility) in comment_ids.iter() {
            let component = PostCommentPath {
                gist: gist.into(),
                username: NAME.into(),
            };

            if visibility == &GistVisibility::Private {
                println!("getting comments; private gist but user==owner");
                let path = V1_API_ROUTES.gist.get_gist_comments(&component);
                let resp = get_request!(&app, &path, cookies.clone());
                assert_eq!(resp.status(), StatusCode::OK);
                let mut comment: Vec<GistComment> = test::read_body_json(resp).await;
                assert_eq!(
                    comment.pop().as_ref().unwrap().comment,
                    comment_payload.comment
                );

                println!("getting comments; private gist but user is unauthenticated");
                let resp = get_request!(&app, &path);
                assert_eq!(resp.status(), ServiceError::GistNotFound.status_code());
                let resp_err: ErrorToResponse = test::read_body_json(resp).await;
                assert_eq!(resp_err.error, format!("{}", ServiceError::GistNotFound));

                println!("getting comments; private gist but user != owner");
                let resp = get_request!(&app, &path, cookies2.clone());
                assert_eq!(resp.status(), ServiceError::GistNotFound.status_code());
                let err: ErrorToResponse = test::read_body_json(resp).await;
                assert_eq!(err.error, format!("{}", ServiceError::GistNotFound));
            } else {
                let path = V1_API_ROUTES.gist.get_gist_comments(&component);
                let resp = get_request!(&app, &path, cookies.clone());
                assert_eq!(resp.status(), StatusCode::OK);
                let mut comment: Vec<GistComment> = test::read_body_json(resp).await;
                assert_eq!(
                    comment.pop().as_ref().unwrap().comment,
                    comment_payload.comment
                );
            }
        }

        /*
         * +++++++++++++++++++++++++++++++++++
         *          DELETE COMMENT
         * +++++++++++++++++++++++++++++++++++
         */

        let mut delete_comment_component = GetCommentPath {
            gist: "gistdoesntexist".into(),
            username: NAME.into(),
            comment_id: 34234234,
        };

        println!("delete comments; unauthenticated, gist does't exist");
        let del_comment_path = V1_API_ROUTES
            .gist
            .get_delete_comment_route(&delete_comment_component);
        let resp = delete_request!(&app, &del_comment_path);
        assert_eq!(resp.status(), StatusCode::FOUND);

        println!("delete comments; authenticated, gist does't exist");
        let resp = delete_request!(&app, &del_comment_path, cookies.clone());
        assert_eq!(resp.status(), ServiceError::GistNotFound.status_code());
        let err: ErrorToResponse = test::read_body_json(resp).await;
        assert_eq!(err.error, format!("{}", ServiceError::GistNotFound));

        println!("delete comments; authenticated, comment doesn't exist");
        delete_comment_component.gist = gist_id.clone();
        let del_comment_path = V1_API_ROUTES
            .gist
            .get_delete_comment_route(&delete_comment_component);
        let resp = delete_request!(&app, &del_comment_path, cookies.clone());
        assert_eq!(resp.status(), ServiceError::CommentNotFound.status_code());
        let err: ErrorToResponse = test::read_body_json(resp).await;
        assert_eq!(err.error, format!("{}", ServiceError::CommentNotFound));

        println!("delete comments; authenticated but comment_owner != user and gist is public");
        delete_comment_component.gist = gist_id.clone();
        delete_comment_component.comment_id = comment_ids.get(0).as_ref().unwrap().0.id;
        let del_comment_path = V1_API_ROUTES
            .gist
            .get_delete_comment_route(&delete_comment_component);
        let resp = delete_request!(&app, &del_comment_path, cookies2.clone());
        assert_eq!(
            resp.status(),
            ServiceError::UnauthorizedOperation("".into()).status_code()
        );
        let err: ErrorToResponse = test::read_body_json(resp).await;
        assert!(err.error.contains(&format!(
            "{}",
            ServiceError::UnauthorizedOperation("".into())
        )));

        println!("delete comments; authenticated but comment_owner != user and gist is private");
        delete_comment_component.gist = private.clone();
        delete_comment_component.comment_id = comment_ids.last().as_ref().unwrap().0.id;
        let del_comment_path = V1_API_ROUTES
            .gist
            .get_delete_comment_route(&delete_comment_component);
        let resp = delete_request!(&app, &del_comment_path, cookies2.clone());
        assert_eq!(resp.status(), ServiceError::GistNotFound.status_code());
        let err: ErrorToResponse = test::read_body_json(resp).await;
        assert_eq!(err.error, format!("{}", ServiceError::GistNotFound));

        println!("delete comments; authenticated  comment_owner == owner");
        let resp = delete_request!(&app, &del_comment_path, cookies.clone());
        assert_eq!(resp.status(), StatusCode::OK);

        /*
         *
         * ============================================
         *                  Gist index
         * ============================================
         *
         */
        let mut gist_index = PostCommentPath {
            username: NAME.into(),
            gist: "non-existant".into(),
        };

        // unauthenticated request
        let path = V1_API_ROUTES.gist.get_gist_index(&gist_index);
        let resp = get_request!(&app, &path);
        assert_eq!(resp.status(), StatusCode::FOUND);

        // non-existant gist
        let path = V1_API_ROUTES.gist.get_gist_index(&gist_index);
        let resp = get_request!(&app, &path, cookies.clone());
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        let err: ErrorToResponse = test::read_body_json(resp).await;
        assert_eq!(err.error, format!("{}", ServiceError::GistNotFound));

        // get private gist with user that doesn't have access
        gist_index.gist = private.clone();
        let path = V1_API_ROUTES.gist.get_gist_index(&gist_index);
        let resp = get_request!(&app, &path, cookies2.clone());
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        let err: ErrorToResponse = test::read_body_json(resp).await;
        assert_eq!(err.error, format!("{}", ServiceError::GistNotFound));

        // get private gist with user=owner
        gist_index.gist = private.clone();
        let path = V1_API_ROUTES.gist.get_gist_index(&gist_index);
        let resp = get_request!(&app, &path, cookies.clone());
        assert_eq!(resp.status(), StatusCode::OK);
        let preview: GistInfo = test::read_body_json(resp).await;
        assert_eq!(preview.owner, NAME);
        assert_eq!(preview.files.len(), files.len());
        for file in preview.files.iter() {
            let processed: Vec<FileInfo> = files
                .iter()
                .map(|f| FileInfo {
                    filename: escape_spaces(&f.filename),
                    content: f.content.clone(),
                })
                .collect();

            assert!(processed
                .iter()
                .any(|f| f.filename == file.filename && f.content == file.content));
        }
    }
}
