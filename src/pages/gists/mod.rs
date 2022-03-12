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
use actix_web::*;

pub use super::{
    auth_ctx, context, errors::*, get_auth_middleware, Footer, TemplateFile, PAGES, PAYLOAD_KEY,
    TEMPLATES,
};

pub mod new;
#[cfg(test)]
mod tests;
pub mod view;

pub const GIST_BASE: TemplateFile = TemplateFile::new("gistbase", "pages/gists/base.html");
pub const GIST_EXPLORE: TemplateFile =
    TemplateFile::new("gist_explore", "pages/gists/explore.html");

pub fn register_templates(t: &mut tera::Tera) {
    for template in [GIST_BASE, GIST_EXPLORE].iter() {
        template.register(t).expect(template.name);
    }
    new::register_templates(t);
    view::register_templates(t);
}

pub fn services(cfg: &mut web::ServiceConfig) {
    new::services(cfg);
    view::services(cfg);
}
