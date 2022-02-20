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

pub mod filemap;
pub mod static_files;

pub use filemap::FileMap;
pub use routes::{Assets, ASSETS};

pub fn services(cfg: &mut web::ServiceConfig) {
    cfg.service(static_files::static_files);
}

pub mod routes {
    use lazy_static::lazy_static;
    use serde::*;

    use super::*;

    lazy_static! {
        pub static ref ASSETS: Assets = Assets::new();
    }

    #[derive(Serialize)]
    /// Top-level routes data structure for V1 AP1
    pub struct Assets {
        /// Authentication routes
        pub css: &'static str,
    }

    impl Assets {
        /// create new instance of Routes
        pub fn new() -> Assets {
            Assets {
                css: &static_files::assets::CSS,
            }
        }
    }
}
