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
use crate::settings::Settings;
use db_core::prelude::*;

pub type BoxDB = Box<dyn GistDatabase>;

pub mod pg {

    use super::*;
    use db_sqlx_postgres::{ConnectionOptions, Fresh};
    use sqlx::postgres::PgPoolOptions;

    pub async fn get_data(pool: Option<u32>) -> BoxDB {
        let settings = Settings::new().unwrap();
        let pool = pool.unwrap_or(settings.database.pool);
        let pool_options = PgPoolOptions::new().max_connections(pool);
        let connection_options = ConnectionOptions::Fresh(Fresh {
            pool_options,
            url: settings.database.url,
        });
        let db = Box::new(connection_options.connect().await.unwrap());

        db
    }
}

pub mod sqlite {
    use super::*;
    use db_sqlx_sqlite::{ConnectionOptions, Fresh};
    use sqlx::sqlite::SqlitePoolOptions;

    pub async fn get_data(pool: Option<u32>) -> BoxDB {
        let settings = Settings::new().unwrap();

        let pool = pool.unwrap_or(settings.database.pool);
        let pool_options = SqlitePoolOptions::new().max_connections(pool);
        let connection_options = ConnectionOptions::Fresh(Fresh {
            pool_options,
            url: settings.database.url,
        });

        let db = Box::new(connection_options.connect().await.unwrap());
        db
    }
}
