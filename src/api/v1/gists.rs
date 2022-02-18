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

use crate::data::api::v1::gists::{CreateGist, FileInfo, GistID};
use crate::errors::*;
use crate::*;

//#[derive(Serialize, Deserialize, Debug, Clone)]
//pub struct File {
//    pub filename: String,
//    pub content: ContentType,
//}

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
            description: self.description.as_ref().map(|s| s.as_str()),
            visibility: &self.visibility,
        }
    }
}

pub fn services(cfg: &mut web::ServiceConfig) {
    cfg.service(new);
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
    Ok(HttpResponse::TemporaryRedirect()
        .insert_header((http::header::LOCATION, gist.id))
        .finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::api::v1::gists::{ContentType, FileType};
    use crate::tests::*;

    #[actix_rt::test]
    async fn test_new_gist_works() {
        let config = [
            sqlx_postgres::get_data().await,
            sqlx_sqlite::get_data().await,
        ];

        for (db, data) in config.iter() {
            const NAME: &str = "httpgisttestuser";
            const EMAIL: &str = "httpgisttestuser@sss.com";
            const PASSWORD: &str = "longpassword2";

            let _ = futures::join!(data.delete_user(db, NAME, PASSWORD),);

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
                    .cookie(cookies)
                    .to_request(),
            )
            .await;
            assert_eq!(create_gist_resp.status(), StatusCode::TEMPORARY_REDIRECT);

            let gist_id = create_gist_resp
                .headers()
                .get(http::header::LOCATION)
                .unwrap()
                .to_str()
                .unwrap();
            data.gist_created_test_helper(db, gist_id, NAME).await;
            data.gist_files_written_helper(db, gist_id, &files).await;
        }
    }
}
