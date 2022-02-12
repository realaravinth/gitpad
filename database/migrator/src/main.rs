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
use std::env;

use sqlx::migrate::MigrateDatabase;
use sqlx::postgres::PgPoolOptions;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::Sqlite;

#[cfg(not(tarpaulin_include))]
#[actix_rt::main]
async fn main() {
    //TODO featuregate sqlite and postgres
    postgres_migrate().await;
    sqlite_migrate().await;
}

async fn postgres_migrate() {
    let db_url = env::var("POSTGRES_DATABASE_URL").expect("set POSTGRES_DATABASE_URL env var");
    let db = PgPoolOptions::new()
        .max_connections(2)
        .connect(&db_url)
        .await
        .expect("Unable to form database pool");

    sqlx::migrate!("../db-sqlx-postgres/migrations/")
        .run(&db)
        .await
        .unwrap();
}

async fn sqlite_migrate() {
    let db_url = env::var("SQLITE_DATABASE_URL").expect("Set SQLITE_DATABASE_URL env var");

    if !matches!(Sqlite::database_exists(&db_url).await, Ok(true)) {
        Sqlite::create_database(&db_url).await.unwrap();
    }

    let db = SqlitePoolOptions::new()
        .max_connections(2)
        .connect(&db_url)
        .await
        .expect("Unable to form database pool");

    sqlx::migrate!("../db-sqlx-sqlite/migrations/")
        .run(&db)
        .await
        .unwrap();
}
