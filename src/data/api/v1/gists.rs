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
use std::path::Path;

use db_core::prelude::*;
use git2::*;
use tokio::fs;

use super::*;
use crate::errors::*;
use crate::utils::*;
use crate::*;

pub struct Gist {
    pub id: String,
    pub repository: git2::Repository,
}

pub struct CreateGist<'a>{
    pub owner: &'a str,
    pub description: Option<&'a str>,
    pub visibility: &'a GistVisibility,
 }

impl Data {
    pub async fn new_gist<T: GistDatabase>(&self, db: &T, msg: &CreateGist<'_>) -> ServiceResult<Gist> {
        loop {
            let gist_id = get_random(32);

            if db.gist_exists(&gist_id).await? {
                continue;
            }

            let gist_path = Path::new(&self.settings.repository.root).join(&gist_id);

            if gist_path.exists() {
                if Repository::open(&gist_path).is_ok() {
                    continue;
                }
                fs::remove_dir_all(&gist_path).await?;
            }

            let create_gist = db_core::CreateGist {
                owner: msg.owner,
                description: msg.description,
                visibility: msg.visibility,
                public_id: &gist_id,
            };

            db.new_gist(&create_gist).await.unwrap();

            fs::create_dir(&gist_path).await?;
            return Ok(Gist {
                id: gist_id,
                repository: Repository::init_bare(&gist_path).unwrap(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::*;

    #[actix_rt::test]
    async fn test_new_gist_works() {
        let config = [
            sqlx_postgres::get_data().await,
            sqlx_sqlite::get_data().await,
        ];

        for (db, data) in config.iter() {

            const NAME: &str = "gisttestuser";
            const EMAIL: &str = "gisttestuser@sss.com";
            const PASSWORD: &str = "longpassword2";

            let _ = futures::join!(
                data.delete_user(db, NAME, PASSWORD),
            );

            let _ = data.register_and_signin(db, NAME, EMAIL, PASSWORD).await;


            let create_gist_msg = CreateGist {
                owner: NAME,
                description: None,
                visibility: &GistVisibility::Public,
            };
            let gist = data.new_gist(db, &create_gist_msg).await.unwrap();
            let path = Path::new(&data.settings.repository.root).join(&gist.id);
            assert!(path.exists());
            assert!(db.gist_exists(&gist.id).await.unwrap());
            let repo = Repository::open(&path).unwrap();
            assert!(repo.is_bare());
            assert!(repo.is_empty().unwrap());
        }
    }
}
