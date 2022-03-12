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
use num_enum::FromPrimitive;
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::errors::*;
use crate::utils::*;
use crate::*;

use super::render_html::{GenerateHTML, SourcegraphQuery};

/// A FileMode represents the kind of tree entries used by git. It
/// resembles regular file systems modes, although FileModes are
/// considerably simpler (there are not so many), and there are some,
/// like Submodule that has no file system equivalent.
// Adapted from https://github.com/go-git/go-git/blob/master/plumbing/filemode/filemode.go(Apache-2.0 License)
#[derive(Debug, PartialEq, Clone, FromPrimitive)]
#[repr(isize)]
pub enum GitFileMode {
    /// Empty is used as the GitFileMode of tree elements when comparing
    /// trees in the following situations:
    ///
    /// - the mode of tree elements before their creation.  
    /// - the mode of tree elements after their deletion.  
    /// - the mode of unmerged elements when checking the index.
    ///
    /// Empty has no file system equivalent.  As Empty is the zero value
    /// of [GitFileMode]
    Empty = 0,
    /// Regular represent non-executable files.
    Regular = 0o100644,
    /// Dir represent a Directory.
    Dir = 0o40000,
    /// Deprecated represent non-executable files with the group writable bit set.  This mode was
    /// supported by the first versions of git, but it has been deprecated nowadays.  This
    /// library(github.com/go-git/go-git uses it, not realaravinth/gitpad at the moment) uses them
    /// internally, so you can read old packfiles, but will treat them as Regulars when interfacing
    /// with the outside world.  This is the standard git behaviour.
    Deprecated = 0o100664,
    /// Executable represents executable files.
    Executable = 0o100755,
    /// Symlink represents symbolic links to files.
    Symlink = 0o120000,
    /// Submodule represents git submodules.  This mode has no file system
    /// equivalent.
    Submodule = 0o160000,

    /// Unsupported file mode
    #[num_enum(default)]
    Unsupported = -1,
}

impl From<&'_ TreeEntry<'_>> for GitFileMode {
    fn from(t: &TreeEntry) -> Self {
        GitFileMode::from(t.filemode() as isize)
    }
}

impl From<TreeEntry<'_>> for GitFileMode {
    fn from(t: TreeEntry) -> Self {
        GitFileMode::from(t.filemode() as isize)
    }
}

pub struct Gist {
    pub id: String,
    pub repository: git2::Repository,
}

pub struct CreateGist<'a> {
    pub owner: &'a str,
    pub description: Option<&'a str>,
    pub visibility: &'a GistVisibility,
}

pub enum GistID<'a> {
    Repository(&'a mut git2::Repository),
    ID(&'a str),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FileInfo {
    pub filename: String,
    pub content: FileType,
}

impl GenerateHTML for FileInfo {
    fn generate(&mut self) {
        fn highlight(code: &mut String, filepath: &str) {
            let q = SourcegraphQuery { code, filepath };
            *code = q.syntax_highlight();
        }

        fn extract(f: &mut FileInfo) {
            match f.content {
                FileType::File(ref mut c) => match &mut *c {
                    ContentType::Binary(_) => (),
                    ContentType::Text(ref mut code) => highlight(&mut *code, &f.filename),
                },

                FileType::Dir(ref mut files) => {
                    for file in files.iter_mut() {
                        extract(file)
                    }
                }
            }
        }

        extract(self);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GistInfo {
    pub files: Vec<FileInfo>,
    pub description: Option<String>,
    pub owner: String,
    pub created: i64,
    pub updated: i64,
    pub visibility: GistVisibility,
    pub id: String,
}

#[derive(Serialize, PartialEq, Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    Binary(Vec<u8>),
    Text(String),
}

impl ContentType {
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Text(text) => text.as_bytes(),
            Self::Binary(bin) => bin.as_ref(),
        }
    }

    pub fn from_blob(blob: &git2::Blob) -> Self {
        if blob.is_binary() {
            Self::Binary(blob.content().to_vec())
        } else {
            Self::Text(String::from_utf8_lossy(blob.content()).to_string())
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FileType {
    /// Contains file content
    File(ContentType),
    Dir(Vec<FileInfo>),
}

impl Data {
    pub async fn new_gist<T: GPDatabse>(
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

    pub(crate) fn get_gist_id_from_repo_path(&self, gist_id: &GistID<'_>) -> String {
        match gist_id {
            GistID::ID(p) => p.to_string(),
            GistID::Repository(r) => {
                let path = r.path().to_path_buf(); // /path/to/repository/.git
                path.file_name().unwrap().to_string_lossy().to_string()
            }
        }
    }

    pub async fn write_file<T: GPDatabse>(
        &self,
        _db: &T,
        gist_id: &mut GistID<'_>,
        files: &[FileInfo],
    ) -> ServiceResult<()> {
        if files.is_empty() {
            return Err(ServiceError::GistEmpty);
        }

        // TODO change updated in DB
        let inner = |repo: &mut Repository| -> ServiceResult<()> {
            let mut tree_builder = match repo.head() {
                Err(_) => repo.treebuilder(None).unwrap(),

                Ok(h) => repo.treebuilder(Some(&h.peel_to_tree().unwrap())).unwrap(),
            };

            let odb = repo.odb().unwrap();

            for file in files.iter() {
                let escaped_filename = escape_spaces(&file.filename);

                match &file.content {
                    FileType::Dir(_dir_contents) => unimplemented!(),
                    FileType::File(f) => {
                        let obj = odb.write(ObjectType::Blob, f.as_bytes()).unwrap();
                        tree_builder
                            .insert(&escaped_filename, obj, 0o100644)
                            .unwrap();
                    }
                }
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
        };

        match gist_id {
            GistID::ID(path) => {
                let mut repo = git2::Repository::open(self.get_repository_path(path)).unwrap();
                inner(&mut repo)
            }
            GistID::Repository(repository) => inner(*repository),
        }
    }

    /// Please note that this method expects path to not contain any spaces
    /// Use [escape_spaces] before calling this method
    ///
    /// For example, a read request for "foo bar.md" will fail even if that file is present
    /// in the repository. However, it will succeed if the output of [escape_spaces] is
    /// used in the request.
    pub async fn read_file<T: GPDatabse>(
        &self,
        _db: &T,
        gist_id: &GistID<'_>,
        path: &str,
    ) -> ServiceResult<FileInfo> {
        let inner = |repo: &git2::Repository| -> ServiceResult<FileInfo> {
            let head = repo.head().unwrap();
            let tree = head.peel_to_tree().unwrap();
            let entry = tree.get_path(Path::new(path)).unwrap();
            fn read_file(id: Oid, repo: &git2::Repository) -> FileType {
                let blob = repo.find_blob(id).unwrap();
                FileType::File(ContentType::from_blob(&blob))
            }

            fn read_dir(id: Oid, repo: &Repository) -> FileType {
                let tree = repo.find_tree(id).unwrap();
                let mut items = Vec::with_capacity(tree.len());
                for item in tree.iter() {
                    if let Some(name) = item.name() {
                        #[allow(clippy::needless_borrow)]
                        let mode: GitFileMode = (&item).into();
                        let file = match mode {
                            GitFileMode::Dir => read_dir(item.id(), repo),
                            GitFileMode::Submodule => unimplemented!(),
                            GitFileMode::Empty => unimplemented!(),
                            GitFileMode::Deprecated => unimplemented!(),
                            GitFileMode::Unsupported => unimplemented!(),
                            GitFileMode::Symlink => unimplemented!(),
                            GitFileMode::Executable => read_file(item.id(), repo),
                            GitFileMode::Regular => read_file(item.id(), repo),
                        };
                        items.push(FileInfo {
                            filename: name.to_owned(),
                            content: file,
                        });
                    }
                }
                FileType::Dir(items)
            }
            let mode: GitFileMode = entry.clone().into();
            if let Some(name) = entry.name() {
                let file = match mode {
                    GitFileMode::Dir => read_dir(entry.id(), repo),
                    GitFileMode::Submodule => unimplemented!(),
                    GitFileMode::Empty => unimplemented!(),
                    GitFileMode::Deprecated => unimplemented!(),
                    GitFileMode::Unsupported => unimplemented!(),
                    GitFileMode::Symlink => unimplemented!(),
                    GitFileMode::Executable => read_file(entry.id(), repo),
                    GitFileMode::Regular => read_file(entry.id(), repo),
                };
                Ok(FileInfo {
                    filename: name.to_string(),
                    content: file,
                })
            } else {
                unimplemented!();
            }
        };

        match gist_id {
            GistID::ID(path) => {
                let repo = git2::Repository::open(self.get_repository_path(path)).unwrap();
                inner(&repo)
            }
            GistID::Repository(repository) => inner(repository),
        }
    }

    /// fetches gist metadata from DB and retrieves contents of all the files stored
    /// in the repository
    // TODO
    // Data::gist_preview uses Data::read_file under the hood, which
    // currently reads subdirectories up to level 1 depth. Decision has
    // to be made regarding what to do with level 2 and below subdirectories.
    pub async fn gist_preview<T: GPDatabse>(
        &self,
        db: &T,
        gist_id: &mut GistID<'_>,
    ) -> ServiceResult<GistInfo> {
        async fn inner<F: GPDatabse>(
            gist_id: &mut GistID<'_>,
            data: &Data,
            db: &F,
        ) -> ServiceResult<Vec<FileInfo>> {
            match &gist_id {
                GistID::Repository(repo) => {
                    let head = repo.head().unwrap();
                    let tree = head.peel_to_tree().unwrap();
                    let mut files = Vec::with_capacity(5);
                    for item in tree.iter() {
                        if let Some(name) = item.name() {
                            let file = data.read_file(db, gist_id, name).await?;
                            files.push(file);
                        }
                    }
                    Ok(files)
                }
                _ => unimplemented!(),
            }
        }

        let gist_public_id = self.get_gist_id_from_repo_path(gist_id);
        let gist_info = db.get_gist(&gist_public_id).await?;

        let files = match &gist_id {
            GistID::ID(path) => {
                let mut repo = git2::Repository::open(self.get_repository_path(path)).unwrap();
                let mut gist_id = GistID::Repository(&mut repo);
                inner(&mut gist_id, self, db).await?
            }
            GistID::Repository(_) => inner(gist_id, self, db).await?,
        };

        let resp = GistInfo {
            created: gist_info.created,
            updated: gist_info.updated,
            files,
            visibility: gist_info.visibility,
            description: gist_info.description,
            owner: gist_info.owner,
            id: gist_info.public_id,
        };

        Ok(resp)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::tests::*;

    impl Data {
        pub async fn gist_created_test_helper<T: GPDatabse>(
            &self,
            db: &T,
            gist_id: &str,
            owner: &str,
        ) {
            let path = self.get_repository_path(gist_id);
            assert!(path.exists());
            assert!(db.gist_exists(gist_id).await.unwrap());
            let repo = Repository::open(&path).unwrap();
            assert!(repo.is_bare());
            assert_eq!(db.get_gist(gist_id).await.unwrap().owner, owner);
        }

        pub async fn gist_files_written_helper<T: GPDatabse>(
            &self,
            db: &T,
            gist_id: &str,
            files: &[FileInfo],
        ) {
            for file in files.iter() {
                let content = self
                    .read_file(db, &GistID::ID(gist_id), &escape_spaces(&file.filename))
                    .await
                    .unwrap();
                let req_escaped_file = FileInfo {
                    filename: escape_spaces(&file.filename),
                    content: file.content.clone(),
                };
                assert_eq!(&content, &req_escaped_file);
            }
        }
    }

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
            const FILE_CONTENT: &str = "foobar";

            let _ = futures::join!(data.delete_user(db, NAME, PASSWORD),);

            let _ = data.register_and_signin(db, NAME, EMAIL, PASSWORD).await;

            let create_gist_msg = CreateGist {
                owner: NAME,
                description: None,
                visibility: &GistVisibility::Public,
            };
            let mut gist = data.new_gist(db, &create_gist_msg).await.unwrap();
            assert!(gist.repository.is_empty().unwrap());
            data.gist_created_test_helper(db, &gist.id, NAME).await;

            // save  files
            let files = [
                FileInfo {
                    filename: "foo".into(),
                    content: FileType::File(ContentType::Text(FILE_CONTENT.into())),
                },
                FileInfo {
                    filename: "bar".into(),
                    content: FileType::File(ContentType::Text(FILE_CONTENT.into())),
                },
                FileInfo {
                    filename: "foo bar".into(),
                    content: FileType::File(ContentType::Text(FILE_CONTENT.into())),
                },
            ];

            data.write_file(db, &mut GistID::Repository(&mut gist.repository), &files)
                .await
                .unwrap();
            data.gist_files_written_helper(db, &gist.id, &files).await;
            let files2 = [FileInfo {
                filename: "notfirstcommit".into(),
                content: FileType::File(ContentType::Text(FILE_CONTENT.into())),
            }];

            data.write_file(db, &mut GistID::ID(&gist.id), &files2)
                .await
                .unwrap();
            data.gist_files_written_helper(db, &gist.id, &files2).await;

            let mut repo = Repository::open(data.get_repository_path(&gist.id)).unwrap();
            assert_eq!(
                data.get_gist_id_from_repo_path(&GistID::Repository(&mut repo)),
                gist.id
            );
            let preview = data
                .gist_preview(db, &mut GistID::ID(&gist.id))
                .await
                .unwrap();
            assert_eq!(preview.owner, NAME);
            assert_eq!(preview.files.len(), 4);
            for file in preview.files.iter() {
                if file.filename == escape_spaces(&files2[0].filename) {
                    assert_eq!(file.content, files2[0].content);
                } else {
                    let processed: Vec<FileInfo> = files
                        .iter()
                        .map(|f| FileInfo {
                            filename: escape_spaces(&f.filename),
                            content: f.content.clone(),
                        })
                        .collect();

                    assert!(processed
                        .iter()
                        .any(|f| f.filename == file.filename && f.content == file.content));
                }
            }
        }
    }
}
