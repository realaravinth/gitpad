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

use crate::settings::Settings;
use crate::AppData;

pub use super::*;

pub struct Login;

pub const LOGIN: &str = "login";

impl Login {
    pub fn page(s: &Settings) -> String {
        let ctx = context(s);
        TEMPLATES.render(LOGIN, &ctx).unwrap()
    }
}

#[my_codegen::get(path = "PAGES.auth.login")]
pub async fn get_login(data: AppData) -> impl Responder {
    let login = Login::page(&data.settings);
    let html = ContentType::html();
    HttpResponse::Ok().content_type(html).body(login)
}

pub fn services(cfg: &mut web::ServiceConfig) {
    cfg.service(get_login);
}

//#[post(path = "PAGES.auth.login")]
//pub async fn login_submit(
//    id: Identity,
//    payload: web::Form<runners::Login>,
//    data: AppData,
//) -> PageResult<impl Responder> {
//    let payload = payload.into_inner();
//    match runners::login_runner(&payload, &data).await {
//        Ok(username) => {
//            id.remember(username);
//            Ok(HttpResponse::Found()
//                .insert_header((header::LOCATION, PAGES.home))
//                .finish())
//        }
//        Err(e) => {
//            let status = e.status_code();
//            let heading = status.canonical_reason().unwrap_or("Error");
//
//            Ok(HttpResponseBuilder::new(status)
//                .content_type("text/html; charset=utf-8")
//                .body(
//                    IndexPage::new(heading, &format!("{}", e))
//                        .render_once()
//                        .unwrap(),
//                ))
//        }
//    }
//}
//
//#[cfg(test)]
//mod tests {
//    use actix_web::test;
//
//    use super::*;
//
//    use crate::api::v1::auth::runners::{Login, Register};
//    use crate::data::Data;
//    use crate::tests::*;
//    use crate::*;
//    use actix_web::http::StatusCode;
//
//    #[actix_rt::test]
//    async fn auth_form_works() {
//        let data = Data::new().await;
//        const NAME: &str = "testuserform";
//        const PASSWORD: &str = "longpassword";
//
//        let app = get_app!(data).await;
//
//        delete_user(NAME, &data).await;
//
//        // 1. Register with email == None
//        let msg = Register {
//            username: NAME.into(),
//            password: PASSWORD.into(),
//            confirm_password: PASSWORD.into(),
//            email: None,
//        };
//        let resp = test::call_service(
//            &app,
//            post_request!(&msg, V1_API_ROUTES.auth.register).to_request(),
//        )
//        .await;
//        assert_eq!(resp.status(), StatusCode::OK);
//
//        // correct form login
//        let msg = Login {
//            login: NAME.into(),
//            password: PASSWORD.into(),
//        };
//
//        let resp = test::call_service(
//            &app,
//            post_request!(&msg, PAGES.auth.login, FORM).to_request(),
//        )
//        .await;
//        assert_eq!(resp.status(), StatusCode::FOUND);
//        let headers = resp.headers();
//        assert_eq!(headers.get(header::LOCATION).unwrap(), PAGES.home,);
//
//        // incorrect form login
//        let msg = Login {
//            login: NAME.into(),
//            password: NAME.into(),
//        };
//        let resp = test::call_service(
//            &app,
//            post_request!(&msg, PAGES.auth.login, FORM).to_request(),
//        )
//        .await;
//        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
//
//        // non-existent form login
//        let msg = Login {
//            login: PASSWORD.into(),
//            password: PASSWORD.into(),
//        };
//        let resp = test::call_service(
//            &app,
//            post_request!(&msg, PAGES.auth.login, FORM).to_request(),
//        )
//        .await;
//        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
//    }
//}
//
