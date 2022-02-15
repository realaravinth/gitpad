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
use std::path::{Path, PathBuf};

use db_core::prelude::*;
use git2::*;
use serde::{Deserialize, Serialize};
use tokio::fs;

use super::*;
use crate::errors::*;
use crate::utils::*;
use crate::*;

pub struct Gist {
    pub id: String,
    pub repository: git2::Repository,
}

pub struct CreateGist<'a> {
    pub owner: &'a str,
    pub description: Option<&'a str>,
    pub visibility: &'a GistVisibility,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct File {
    pub filename: String,
    pub content: String,
}

impl Data {
    pub async fn new_gist<T: GistDatabase>(
        &self,
        db: &T,
        msg: &CreateGist<'_>,
    ) -> ServiceResult<Gist> {
        loop {
            let gist_id = get_random(32);

            if db.gist_exists(&gist_id).await? {
                continue;
            }

            let gist_path = self.get_repository_path(&gist_id);

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

    pub(crate) fn get_repository_path(&self, gist_id: &str) -> PathBuf {
        Path::new(&self.settings.repository.root).join(gist_id)
    }

    pub async fn write_file<T: GistDatabase>(
        &self,
        _db: &T,
        gist_id: &str,
        files: &[File],
    ) -> ServiceResult<()> {
        // TODO change updated in DB

        let repo = git2::Repository::open(self.get_repository_path(gist_id)).unwrap();
        let mut tree_builder = repo.treebuilder(None).unwrap();
        let odb = repo.odb().unwrap();

        for file in files.iter() {
            let escaped_filename = escape_spaces(&file.filename);

            let obj = odb
                .write(ObjectType::Blob, file.content.as_bytes())
                .unwrap();
            tree_builder
                .insert(&escaped_filename, obj, 0o100644)
                .unwrap();
        }

        let tree_hash = tree_builder.write().unwrap();
        let author = Signature::now("gists", "admin@gists.batsense.net").unwrap();
        let committer = Signature::now("gists", "admin@gists.batsense.net").unwrap();

        let commit_tree = repo.find_tree(tree_hash).unwrap();
        let msg = "";
        if let Err(e) = repo.head() {
            if e.code() == ErrorCode::UnbornBranch && e.class() == ErrorClass::Reference {
                // fisrt commit ever; set parent commit(s) to empty array
                repo.commit(Some("HEAD"), &author, &committer, msg, &commit_tree, &[])
                    .unwrap();
            } else {
                panic!("{:?}", e);
            }
        } else {
            let head_ref = repo.head().unwrap();
            let head_commit = head_ref.peel_to_commit().unwrap();
            repo.commit(
                Some("HEAD"),
                &author,
                &committer,
                msg,
                &commit_tree,
                &[&head_commit],
            )
            .unwrap();
        };

        Ok(())
    }

    /// Please note that this method expects path to not contain any spaces
    /// Use [escape_spaces] before calling this method
    ///
    /// For example, a read request for "foo bar.md" will fail even if that file is present
    /// in the repository. However, it will succeed if the output of [escape_spaces] is
    /// used in the request.
    pub async fn read_file<T: GistDatabase>(
        &self,
        _db: &T,
        gist_id: &str,
        path: &str,
    ) -> ServiceResult<Vec<u8>> {
        let repo = git2::Repository::open(self.get_repository_path(gist_id)).unwrap();
        let head = repo.head().unwrap();
        let tree = head.peel_to_tree().unwrap();
        let entry = tree.get_path(Path::new(path)).unwrap();
        let blob = repo.find_blob(entry.id()).unwrap();
        Ok(blob.content().to_vec())
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

            let _ = futures::join!(data.delete_user(db, NAME, PASSWORD),);

            let _ = data.register_and_signin(db, NAME, EMAIL, PASSWORD).await;

            let create_gist_msg = CreateGist {
                owner: NAME,
                description: None,
                visibility: &GistVisibility::Public,
            };
            let gist = data.new_gist(db, &create_gist_msg).await.unwrap();
            let path = data.get_repository_path(&gist.id);
            assert!(path.exists());
            assert!(db.gist_exists(&gist.id).await.unwrap());
            let repo = Repository::open(&path).unwrap();
            assert!(repo.is_bare());
            assert!(repo.is_empty().unwrap());

            // save  files
            let files = [
                File {
                    filename: "foo".into(),
                    content: "foobar".into(),
                },
                File {
                    filename: "bar".into(),
                    content: "foobar".into(),
                },
                File {
                    filename: "foo bar".into(),
                    content: "foobar".into(),
                },
            ];

            data.write_file(db, &gist.id, &files).await.unwrap();
            for file in files.iter() {
                let content = data
                    .read_file(db, &gist.id, &escape_spaces(&file.filename))
                    .await
                    .unwrap();
                assert_eq!(String::from_utf8_lossy(&content), file.content);
            }
        }
    }
}
