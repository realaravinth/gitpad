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
//! Demo user: Enable users to try out your application without signing up
use std::time::Duration;

use tokio::spawn;
use tokio::time::sleep;

use crate::data::api::v1::auth::Register;
use crate::db::BoxDB;
use crate::*;

use errors::*;

/// Demo username
pub const DEMO_USER: &str = "aaronsw";
/// Demo password
pub const DEMO_PASSWORD: &str = "password";

/// register demo user runner
async fn register_demo_user(db: &BoxDB, data: &AppData) -> ServiceResult<()> {
    if !data.username_exists(db, DEMO_USER).await?.exists {
        let register_payload = Register {
            username: DEMO_USER.into(),
            password: DEMO_PASSWORD.into(),
            confirm_password: DEMO_PASSWORD.into(),
            email: None,
        };

        log::info!("Registering demo user");
        match data.register(db, &register_payload).await {
            Err(ServiceError::UsernameTaken) | Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    } else {
        Ok(())
    }
}

async fn delete_demo_user(db: &BoxDB, data: &AppData) -> ServiceResult<()> {
    log::info!("Deleting demo user");
    data.delete_user(db, DEMO_USER, DEMO_PASSWORD).await?;
    Ok(())
}

/// creates and deletes demo user periodically
pub async fn run(db: BoxDB, data: AppData, duration: Duration) -> ServiceResult<()> {
    register_demo_user(&db, &data).await?;

    let fut = async move {
        loop {
            sleep(duration).await;
            if let Err(e) = delete_demo_user(&db, &data).await {
                log::error!("Error while deleting demo user: {:?}", e);
            }
            if let Err(e) = register_demo_user(&db, &data).await {
                log::error!("Error while registering demo user: {:?}", e);
            }
        }
    };
    spawn(fut);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::api::v1::auth::Login;
    use crate::tests::*;

    const DURATION: u64 = 5;

    #[actix_rt::test]
    async fn postgrest_demo_works() {
        let (db, data) = sqlx_postgres::get_data().await;
        let (db2, _) = sqlx_postgres::get_data().await;
        demo_account_works(data, &db, &db2).await;
    }

    #[actix_rt::test]
    async fn sqlite_demo_works() {
        let (db, data) = sqlx_sqlite::get_data().await;
        let (db2, _) = sqlx_sqlite::get_data().await;
        demo_account_works(data, &db, &db2).await;
    }

    async fn demo_account_works(data: Arc<Data>, db: &BoxDB, db2: &BoxDB) {
        let _ = data.delete_user(db, DEMO_USER, DEMO_PASSWORD).await;
        let data = AppData::new(data);
        let duration = Duration::from_secs(DURATION);

        // register works
        let _ = register_demo_user(db, &data).await.unwrap();
        assert!(data.username_exists(db, DEMO_USER).await.unwrap().exists);
        let signin = Login {
            login: DEMO_USER.into(),
            password: DEMO_PASSWORD.into(),
        };
        data.login(db, &signin).await.unwrap();

        // deletion works
        assert!(super::delete_demo_user(db, &data).await.is_ok());
        assert!(!data.username_exists(db, DEMO_USER).await.unwrap().exists);
        run(db2.clone(), data.clone(), duration).await.unwrap();

        sleep(Duration::from_secs(DURATION)).await;
        assert!(data.username_exists(db, DEMO_USER).await.unwrap().exists);
    }
}
