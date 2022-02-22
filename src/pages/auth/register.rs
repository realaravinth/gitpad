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
use actix_web::http::header::ContentType;
use std::cell::RefCell;
use tera::Context;

use crate::data::api::v1::auth::Register as RegisterPayload;
use crate::pages::errors::*;
use crate::settings::Settings;
use crate::AppData;

pub use super::*;

pub const REGISTER: &str = "register";

pub struct Register {
    ctx: RefCell<Context>,
}

impl CtxError for Register {
    fn with_error(&self, e: &ReadableError) -> String {
        self.ctx.borrow_mut().insert(ERROR_KEY, e);
        self.render()
    }
}

impl Register {
    fn new(settings: &Settings, payload: Option<&RegisterPayload>) -> Self {
        let ctx = RefCell::new(context(settings));
        if let Some(payload) = payload {
            ctx.borrow_mut().insert(PAYLOAD_KEY, payload);
        }
        Self { ctx }
    }

    pub fn render(&self) -> String {
        TEMPLATES.render(REGISTER, &self.ctx.borrow()).unwrap()
    }

    pub fn page(s: &Settings) -> String {
        let p = Self::new(s, None);
        p.render()
    }
}

#[my_codegen::get(path = "PAGES.auth.register")]
pub async fn get_register(data: AppData) -> impl Responder {
    let login = Register::page(&data.settings);
    let html = ContentType::html();
    HttpResponse::Ok().content_type(html).body(login)
}

pub fn services(cfg: &mut web::ServiceConfig) {
    cfg.service(get_register);
    cfg.service(register_submit);
}

#[my_codegen::post(path = "PAGES.auth.register")]
pub async fn register_submit(
    payload: web::Form<RegisterPayload>,
    data: AppData,
    db: crate::DB,
) -> PageResult<impl Responder, Register> {
    data.register(&(**db), &payload)
        .await
        .map_err(|e| PageError::new(Register::new(&data.settings, Some(&payload)), e))?;
    Ok(HttpResponse::Found()
        .insert_header((http::header::LOCATION, PAGES.auth.login))
        .finish())
}

#[cfg(test)]
mod tests {
    use super::Register;
    use super::RegisterPayload;
    use crate::errors::*;
    use crate::pages::errors::*;
    use crate::settings::Settings;

    #[test]
    fn register_page_renders() {
        let settings = Settings::new().unwrap();
        Register::page(&settings);
        let payload = RegisterPayload {
            username: "foo".into(),
            password: "foo".into(),
            confirm_password: "foo".into(),
            email: Some("foo".into()),
        };
        let page = Register::new(&settings, Some(&payload));
        page.with_error(&ReadableError::new(&ServiceError::WrongPassword));
        page.render();
    }
}
