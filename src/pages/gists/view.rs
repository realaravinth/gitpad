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
use std::cell::RefCell;

use actix_identity::Identity;
use actix_web::http::header::ContentType;
use serde::*;
use tera::Context;

use db_core::prelude::*;

use crate::data::api::v1::gists::{GistID, GistInfo};
use crate::data::api::v1::render_html::GenerateHTML;
use crate::errors::*;
use crate::pages::routes::GistProfilePathComponent;
use crate::pages::routes::PostCommentPath;
use crate::settings::Settings;
use crate::AppData;

pub use super::*;

pub const VIEW_GIST: TemplateFile = TemplateFile::new("viewgist", "pages/gists/view/index.html");
pub const GIST_TEXTFILE: TemplateFile =
    TemplateFile::new("gist_textfile", "pages/gists/view/_text.html");
pub const GIST_FILENAME: TemplateFile =
    TemplateFile::new("gist_filename", "pages/gists/view/_filename.html");

pub const GIST_COMMENT_INPUT: TemplateFile =
    TemplateFile::new("gist_comment_input", "components/comments.html");

pub fn register_templates(t: &mut tera::Tera) {
    for template in [VIEW_GIST, GIST_FILENAME, GIST_TEXTFILE, GIST_COMMENT_INPUT].iter() {
        template.register(t).expect(template.name);
    }
}

pub fn services(cfg: &mut web::ServiceConfig) {
    cfg.service(view_preview);
}

pub struct ViewGist {
    ctx: RefCell<Context>,
}

impl CtxError for ViewGist {
    fn with_error(&self, e: &ReadableError) -> String {
        self.ctx.borrow_mut().insert(ERROR_KEY, e);
        self.render()
    }
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct Payload<'a> {
    pub gist: Option<&'a GistInfo>,
    pub comments: Option<&'a Vec<GistComment>>,
}

impl ViewGist {
    pub fn new(username: Option<&str>, payload: Payload, settings: &Settings) -> Self {
        let mut ctx = auth_ctx(username, settings);
        ctx.insert("visibility_private", GistVisibility::Private.to_str());
        ctx.insert("visibility_unlisted", GistVisibility::Unlisted.to_str());
        ctx.insert("visibility_public", GistVisibility::Public.to_str());

        ctx.insert(PAYLOAD_KEY, &payload);
        if let Some(gist) = payload.gist {
            ctx.insert(
                "gist_owner_link",
                &PAGES.gist.get_profile_route(GistProfilePathComponent {
                    username: &gist.owner,
                }),
            );
        }

        if let Some(comments) = payload.comments {
            ctx.insert("gist_comments", comments);
        }

        let ctx = RefCell::new(ctx);
        Self { ctx }
    }

    pub fn render(&self) -> String {
        TEMPLATES
            .render(VIEW_GIST.name, &self.ctx.borrow())
            .unwrap()
    }

    pub fn page(username: Option<&str>, payload: Payload, s: &Settings) -> String {
        let p = Self::new(username, payload, s);
        p.render()
    }
}
#[my_codegen::get(path = "PAGES.gist.view_gist", wrap = "super::get_auth_middleware()")]
async fn view_preview(
    data: AppData,
    db: crate::DB,
    id: Identity,
    path: web::Path<PostCommentPath>,
) -> PageResult<impl Responder, ViewGist> {
    let username = id.identity();

    let map_err = |e: ServiceError, gist: Option<&GistInfo>| -> PageError<ViewGist> {
        PageError::new(
            ViewGist::new(
                username.as_deref(),
                Payload {
                    gist,
                    comments: None,
                },
                &data.settings,
            ),
            e,
        )
    };

    let gist = db.get_gist(&path.gist).await.map_err(|e| {
        let err: ServiceError = e.into();
        map_err(err, None)
    })?;

    if let Some(username) = &username {
        if gist.visibility == GistVisibility::Private && username != &gist.owner {
            return Err(map_err(ServiceError::GistNotFound, None));
        }
    }

    let mut gist = data
        .gist_preview(db.as_ref(), &mut GistID::ID(&path.gist))
        .await
        .map_err(|e| map_err(e, None))?;

    gist.files.iter_mut().for_each(|file| file.generate());

    log::info!("testing start");

    let comments = db.get_comments_on_gist(&path.gist).await.map_err(|e| {
        let e: ServiceError = e.into();
        map_err(e, None)
    })?;

    log::info!("testing end");

    let ctx = Payload {
        gist: Some(&gist),
        comments: Some(&comments),
    };

    let page = ViewGist::page(username.as_deref(), ctx, &data.settings);
    let html = ContentType::html();
    Ok(HttpResponse::Ok().content_type(html).body(page))
}
