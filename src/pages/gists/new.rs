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

use db_core::GistVisibility;

use crate::data::api::v1::gists::{
    ContentType as GistContentType, CreateGist, FileInfo, FileType, GistID,
};
use crate::errors::*;
use crate::pages::routes::PostCommentPath;
use crate::settings::Settings;
use crate::AppData;

pub use super::*;

pub const NEW_GIST: TemplateFile = TemplateFile::new("newgist", "pages/gists/new/index.html");

pub fn register_templates(t: &mut tera::Tera) {
    NEW_GIST.register(t).expect(NEW_GIST.name);
}

pub fn services(cfg: &mut web::ServiceConfig) {
    cfg.service(new);
}

pub struct NewGist {
    ctx: RefCell<Context>,
}

impl CtxError for NewGist {
    fn with_error(&self, e: &ReadableError) -> String {
        self.ctx.borrow_mut().insert(ERROR_KEY, e);
        self.render()
    }
}

impl NewGist {
    pub fn new(username: &str, settings: &Settings, payload: Option<&[FieldNames<&str>]>) -> Self {
        let ctx = RefCell::new(auth_ctx(username, settings));
        if let Some(payload) = payload {
            ctx.borrow_mut().insert(PAYLOAD_KEY, &payload);
        }
        Self { ctx }
    }

    pub fn render(&self) -> String {
        TEMPLATES.render(NEW_GIST.name, &self.ctx.borrow()).unwrap()
    }

    pub fn page(username: &str, s: &Settings) -> String {
        let p = Self::new(username, s, None);
        p.render()
    }
}

#[my_codegen::get(path = "PAGES.gist.new", wrap = "super::get_auth_middleware()")]
async fn new(data: AppData, id: Identity) -> impl Responder {
    let username = id.identity().unwrap();
    let page = NewGist::page(&username, &data.settings);
    let html = ContentType::html();
    HttpResponse::Ok().content_type(html).body(page)
}

const CONTENT_FIELD_NAME_PREFIX: &str = "content__";
const FILENAME_FIELD_NAME_PREFIX: &str = "filename__";

#[derive(Serialize, PartialEq, Debug, Clone)]
pub struct FieldNames<T: Serialize + ToString> {
    pub filename: T,
    pub content: T,
}

impl<T: Serialize + ToString> From<FieldNames<T>> for FileInfo {
    fn from(f: FieldNames<T>) -> Self {
        FileInfo {
            filename: f.filename.to_string(),
            content: FileType::File(GistContentType::Text(f.content.to_string())),
        }
    }
}

impl<T: Serialize + ToString> FieldNames<T> {
    pub fn new(num: usize) -> FieldNames<String> {
        let filename = format!("{}{}", FILENAME_FIELD_NAME_PREFIX, num);
        let content = format!("{}{}", CONTENT_FIELD_NAME_PREFIX, num);
        FieldNames { filename, content }
    }

    #[allow(clippy::type_complexity)]
    pub fn from_serde_json(
        json: &serde_json::Value,
    ) -> std::result::Result<Vec<FieldNames<&str>>, (Vec<FieldNames<&str>>, ServiceError)> {
        let mut count = 1;
        let mut fields = Self::new(count);
        let mut resp = Vec::default();
        while json.get(&fields.content).is_some() || json.get(&fields.filename).is_some() {
            let content = json.get(&fields.content);
            if content.is_none() {
                return Err((
                    resp,
                    ServiceError::BadRequest(format!("content for {} file is empty", count)),
                ));
            }
            let content = content.unwrap().as_str().unwrap();
            let filename = json.get(&fields.filename);
            if filename.is_none() {
                return Err((
                    resp,
                    ServiceError::BadRequest(format!("filename for {} field is empty", count)),
                ));
            }
            let filename = filename.unwrap().as_str().unwrap();
            resp.push(FieldNames { filename, content });
            count += 1;
            fields = Self::new(count);
        }
        if resp.is_empty() {
            Err((Vec::default(), ServiceError::GistEmpty))
        } else {
            Ok(resp)
        }
    }
}

fn get_visibility(payload: &serde_json::Value) -> ServiceResult<GistVisibility> {
    let visibility = payload.get("visibility");
    if let Some(visibility) = visibility {
        use std::str::FromStr;
        if let Some(visibility) = visibility.as_str() {
            return Ok(GistVisibility::from_str(visibility)?);
        }
    }
    Err(ServiceError::BadRequest("unknown visibility value".into()))
}

fn get_description(payload: &serde_json::Value) -> Option<&str> {
    let description = payload.get("description");
    if let Some(description) = description {
        if let Some(description) = description.as_str() {
            if !description.is_empty() {
                return Some(description);
            }
        }
    }
    None
}
#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn fiefieldname_generation_worksldname_generation_works() {
        use super::*;
        assert_ne!(CONTENT_FIELD_NAME_PREFIX, FILENAME_FIELD_NAME_PREFIX);
        let f: FieldNames<String> = FieldNames::<String>::new(10);
        let f2: FieldNames<String> = FieldNames::<String>::new(11);
        assert_ne!(f2.content, f.content);
        assert_ne!(f2.filename, f.filename);
    }

    #[test]
    fn new_gist_json_extraction_works() {
        use super::*;

        let f1: FieldNames<String> = FieldNames::<String>::new(1);
        let f2: FieldNames<String> = FieldNames::<String>::new(2);
        let f1_content = "file 1 content";
        let f1_name = "1.md";
        let f2_content = "file 2 content";
        let f2_name = "1.md";
        let visibility = GistVisibility::Public;
        let description = "some description";
        let ideal = json!({
            "description": description,
            f1.filename.clone(): f1_name,
            f1.content.clone(): f1_content,
            f2.filename.clone(): f2_name,
            f2.content.clone(): f2_content,
            "visibility": visibility.to_str(),
        });

        let from_json_fieldnames = FieldNames::<&str>::from_serde_json(&ideal).unwrap();
        assert_eq!(from_json_fieldnames.len(), 2);
        for f in from_json_fieldnames.iter() {
            if f.content.contains(f1_content) {
                assert_eq!(f.content, f1_content);
                assert_eq!(f.filename, f1_name);
            } else {
                assert_eq!(f.content, f2_content);
                assert_eq!(f.filename, f2_name);
            }
        }
        assert_eq!(get_visibility(&ideal).unwrap(), visibility);
        assert_eq!(get_description(&ideal), Some(description));

        // empty description
        let empty = serde_json::Value::default();
        assert_eq!(get_description(&empty), None);
        // empty fieldnames
        let empty_gist_err = FieldNames::<&str>::from_serde_json(&empty);
        assert!(empty_gist_err.is_err());
        assert_eq!(
            empty_gist_err.err(),
            Some((Vec::default(), ServiceError::GistEmpty))
        );
        assert_eq!(
            get_visibility(&empty).err(),
            Some(ServiceError::BadRequest("unknown visibility value".into()))
        );

        // partially empty fields
        let partially_empty_files = json!({
            "description": "",
            f1.filename.clone(): f1_name,
        });
        // description is empty sting
        assert_eq!(get_description(&empty), None);
        let empty_gist_err = FieldNames::<&str>::from_serde_json(&partially_empty_files);
        assert!(empty_gist_err.is_err());
        assert_eq!(
            empty_gist_err.err(),
            Some((
                Vec::default(),
                ServiceError::BadRequest("content for 1 file is empty".into())
            ))
        );

        let fields1 = FieldNames {
            filename: f1_name,
            content: f1_content,
        };

        // some partially empty fields
        let some_partially_empty_files = json!({
            f1.filename.clone(): f1_name,
            f1.content.clone(): f1_content,
            f2.filename.clone(): f2_name,
        });
        let some_empty_gist_err = FieldNames::<&str>::from_serde_json(&some_partially_empty_files);
        assert!(some_empty_gist_err.is_err());
        assert_eq!(
            some_empty_gist_err.err(),
            Some((
                vec![fields1.clone()],
                ServiceError::BadRequest("content for 2 file is empty".into())
            ))
        );

        let some_partially_empty_files = json!({
            f1.filename.clone(): f1_name,
            f1.content.clone(): f1_content,
            f2.content.clone(): f2_content,
        });
        let some_empty_gist_err = FieldNames::<&str>::from_serde_json(&some_partially_empty_files);
        assert!(some_empty_gist_err.is_err());
        assert_eq!(
            some_empty_gist_err.err(),
            Some((
                vec![fields1.clone()],
                ServiceError::BadRequest("filename for 2 field is empty".into())
            ))
        );

        // From<FieldNames> for FileInfo
        let f: FileInfo = fields1.clone().into();
        assert_eq!(f.filename, fields1.filename);
        assert_eq!(
            f.content,
            FileType::File(GistContentType::Text(fields1.content.to_owned()))
        );
    }
}
