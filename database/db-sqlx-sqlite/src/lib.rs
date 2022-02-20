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
use db_core::dev::*;
use std::str::FromStr;

use sqlx::sqlite::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::types::time::OffsetDateTime;

pub mod errors;
#[cfg(test)]
pub mod tests;

#[derive(Clone)]
pub struct Database {
    pub pool: SqlitePool,
}

/// Use an existing database pool
pub struct Conn(pub SqlitePool);

/// Connect to databse
pub enum ConnectionOptions {
    /// fresh connection
    Fresh(Fresh),
    /// existing connection
    Existing(Conn),
}

pub struct Fresh {
    pub pool_options: SqlitePoolOptions,
    pub url: String,
}

pub mod dev {
    pub use super::errors::*;
    pub use super::Database;
    pub use db_core::dev::*;
    pub use prelude::*;
    pub use sqlx::Error;
}

pub mod prelude {
    pub use super::*;
    pub use db_core::prelude::*;
}

#[async_trait]
impl Connect for ConnectionOptions {
    type Pool = Database;
    async fn connect(self) -> DBResult<Self::Pool> {
        let pool = match self {
            Self::Fresh(fresh) => fresh
                .pool_options
                .connect(&fresh.url)
                .await
                .map_err(|e| DBError::DBError(Box::new(e)))?,
            Self::Existing(conn) => conn.0,
        };
        Ok(Database { pool })
    }
}

use dev::*;

#[async_trait]
impl Migrate for Database {
    async fn migrate(&self) -> DBResult<()> {
        sqlx::migrate!("./migrations/")
            .run(&self.pool)
            .await
            .map_err(|e| DBError::DBError(Box::new(e)))?;
        Ok(())
    }
}

#[async_trait]
impl GPDatabse for Database {
    async fn email_login(&self, email: &str) -> DBResult<Creds> {
        sqlx::query_as!(
            Creds,
            r#"SELECT username, password  FROM gists_users WHERE email = ($1)"#,
            email,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            Error::RowNotFound => DBError::AccountNotFound,
            e => DBError::DBError(Box::new(e)),
        })
    }

    async fn username_login(&self, username: &str) -> DBResult<Password> {
        sqlx::query_as!(
            Password,
            r#"SELECT password  FROM gists_users WHERE username = ($1)"#,
            username,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            Error::RowNotFound => DBError::AccountNotFound,
            e => DBError::DBError(Box::new(e)),
        })
    }

    async fn email_register(&self, payload: &EmailRegisterPayload) -> DBResult<()> {
        sqlx::query!(
            "insert into gists_users 
        (username , password, email, secret) values ($1, $2, $3, $4)",
            payload.username,
            payload.password,
            payload.email,
            payload.secret,
        )
        .execute(&self.pool)
        .await
        .map_err(map_register_err)?;
        Ok(())
    }

    async fn username_register(&self, payload: &UsernameRegisterPayload) -> DBResult<()> {
        sqlx::query!(
            "INSERT INTO gists_users 
        (username , password,  secret) VALUES ($1, $2, $3)",
            payload.username,
            payload.password,
            payload.secret,
        )
        .execute(&self.pool)
        .await
        .map_err(map_register_err)?;
        Ok(())
    }

    async fn update_email(&self, payload: &UpdateEmailPayload) -> DBResult<()> {
        let x = sqlx::query!(
            "UPDATE gists_users set email = $1
        WHERE username = $2",
            payload.email,
            payload.username,
        )
        .execute(&self.pool)
        .await
        .map_err(map_register_err)?;
        if x.rows_affected() == 0 {
            return Err(DBError::AccountNotFound);
        }
        Ok(())
    }
    async fn update_password(&self, payload: &Creds) -> DBResult<()> {
        let x = sqlx::query!(
            "UPDATE gists_users set password = $1
        WHERE username = $2",
            payload.password,
            payload.username,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DBError::DBError(Box::new(e)))?;
        if x.rows_affected() == 0 {
            return Err(DBError::AccountNotFound);
        }
        Ok(())
    }

    async fn email_exists(&self, email: &str) -> DBResult<bool> {
        match sqlx::query!("SELECT id from gists_users WHERE email = $1", email)
            .fetch_one(&self.pool)
            .await
        {
            Ok(_) => Ok(true),
            Err(Error::RowNotFound) => Ok(false),
            Err(e) => Err(DBError::DBError(Box::new(e))),
        }
    }

    async fn delete_account(&self, username: &str) -> DBResult<()> {
        sqlx::query!("DELETE FROM gists_users WHERE username = ($1)", username,)
            .execute(&self.pool)
            .await
            .map_err(|e| DBError::DBError(Box::new(e)))?;
        Ok(())
    }

    async fn username_exists(&self, username: &str) -> DBResult<bool> {
        match sqlx::query!("SELECT id from gists_users WHERE username = $1", username)
            .fetch_one(&self.pool)
            .await
        {
            Ok(_) => Ok(true),
            Err(Error::RowNotFound) => Ok(false),
            Err(e) => Err(DBError::DBError(Box::new(e))),
        }
    }

    async fn update_username(&self, payload: &UpdateUsernamePayload) -> DBResult<()> {
        let x = sqlx::query!(
            "UPDATE gists_users set username = $1 WHERE username = $2",
            payload.new_username,
            payload.old_username,
        )
        .execute(&self.pool)
        .await
        .map_err(map_register_err)?;
        if x.rows_affected() == 0 {
            return Err(DBError::AccountNotFound);
        }
        Ok(())
    }
    async fn update_secret(&self, username: &str, secret: &str) -> DBResult<()> {
        let x = sqlx::query!(
            "UPDATE gists_users set secret = $1
        WHERE username = $2",
            secret,
            username,
        )
        .execute(&self.pool)
        .await
        .map_err(map_register_err)?;
        if x.rows_affected() == 0 {
            return Err(DBError::AccountNotFound);
        }

        Ok(())
    }

    async fn get_secret(&self, username: &str) -> DBResult<String> {
        struct Secret {
            secret: String,
        }
        let secret = sqlx::query_as!(
            Secret,
            r#"SELECT secret  FROM gists_users WHERE username = ($1)"#,
            username,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            Error::RowNotFound => DBError::AccountNotFound,
            e => DBError::DBError(Box::new(e)),
        })?;

        Ok(secret.secret)
    }

    /// ping DB
    async fn ping(&self) -> bool {
        use sqlx::Connection;

        if let Ok(mut con) = self.pool.acquire().await {
            con.ping().await.is_ok()
        } else {
            false
        }
    }
    /// Check if a Gist with the given ID exists
    async fn gist_exists(&self, public_id: &str) -> DBResult<bool> {
        match sqlx::query!("SELECT ID from gists_gists WHERE public_id = $1", public_id)
            .fetch_one(&self.pool)
            .await
        {
            Ok(_) => Ok(true),
            Err(Error::RowNotFound) => Ok(false),
            Err(e) => Err(DBError::DBError(Box::new(e))),
        }
    }
    /// Create new gists
    async fn new_gist(&self, gist: &CreateGist) -> DBResult<()> {
        let now = now_unix_time_stamp();
        let visibility = gist.visibility.to_str();
        if let Some(description) = &gist.description {
            sqlx::query!(
                "INSERT INTO gists_gists 
        (owner_id , description, public_id, visibility, created, updated)
        VALUES (
            (SELECT ID FROM gists_users WHERE username = $1),
            $2, $3, (SELECT ID FROM gists_visibility WHERE name = $4), $5, $6
        )",
                gist.owner,
                description,
                gist.public_id,
                visibility,
                now,
                now
            )
            .execute(&self.pool)
            .await
            .map_err(map_register_err)?;
        } else {
            sqlx::query!(
                "INSERT INTO gists_gists 
        (owner_id , public_id, visibility, created, updated)
        VALUES (
            (SELECT ID FROM gists_users WHERE username = $1),
            $2, (SELECT ID FROM gists_visibility WHERE name = $3), $4, $5
        )",
                gist.owner,
                gist.public_id,
                visibility,
                now,
                now
            )
            .execute(&self.pool)
            .await
            .map_err(map_register_err)?;
        }
        Ok(())
    }
    /// Retrieve gist from database
    async fn get_gist(&self, public_id: &str) -> DBResult<Gist> {
        let res = sqlx::query_as!(
            InnerGist,
            "SELECT
                owner,
                visibility,
                created,
                updated,
                public_id,
                description
            FROM
                gists_gists_view
            WHERE public_id = $1
            ",
            public_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            Error::RowNotFound => DBError::GistNotFound,
            e => DBError::DBError(Box::new(e)),
        })?;
        res.into_gist()
    }

    /// Retrieve gists belonging to user from database
    async fn get_user_gists(&self, owner: &str) -> DBResult<Vec<Gist>> {
        let mut res = sqlx::query_as!(
            InnerGist,
            "SELECT
                owner,
                visibility,
                created,
                updated,
                public_id,
                description
            FROM
                gists_gists_view
            WHERE owner = $1
            ",
            owner
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| match e {
            Error::RowNotFound => DBError::GistNotFound,
            e => DBError::DBError(Box::new(e)),
        })?;

        let mut gists = Vec::with_capacity(res.len());
        for r in res.drain(..) {
            gists.push(r.into_gist()?);
        }
        Ok(gists)
    }

    /// Retrieve gists belonging to user from database
    async fn get_user_public_gists(&self, owner: &str) -> DBResult<Vec<Gist>> {
        const PUBLIC: &str = GistVisibility::Public.to_str();
        let mut res = sqlx::query_as!(
            InnerGist,
            "SELECT
                owner,
                visibility,
                created,
                updated,
                public_id,
                description
            FROM
                gists_gists_view
            WHERE 
                owner = $1
            AND
                visibility = $2
            ",
            owner,
            PUBLIC
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| match e {
            Error::RowNotFound => DBError::GistNotFound,
            e => DBError::DBError(Box::new(e)),
        })?;

        let mut gists = Vec::with_capacity(res.len());
        for r in res.drain(..) {
            gists.push(r.into_gist()?);
        }
        Ok(gists)
    }

    /// Retrieve gists belonging to user from database
    async fn get_user_public_unlisted_gists(&self, owner: &str) -> DBResult<Vec<Gist>> {
        const PRIVATE: &str = GistVisibility::Private.to_str();
        let mut res = sqlx::query_as!(
            InnerGist,
            "SELECT
                owner,
                visibility,
                created,
                updated,
                public_id,
                description
            FROM
                gists_gists_view
            WHERE 
                owner = $1
            AND
                visibility <> $2
            ",
            owner,
            PRIVATE
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| match e {
            Error::RowNotFound => DBError::GistNotFound,
            e => DBError::DBError(Box::new(e)),
        })?;

        let mut gists = Vec::with_capacity(res.len());
        for r in res.drain(..) {
            gists.push(r.into_gist()?);
        }
        Ok(gists)
    }

    async fn delete_gist(&self, owner: &str, public_id: &str) -> DBResult<()> {
        sqlx::query!(
            "DELETE FROM gists_gists 
        WHERE 
            public_id = $1
        AND
            owner_id = (SELECT ID FROM gists_users WHERE username = $2)
        ",
            public_id,
            owner
        )
        .execute(&self.pool)
        .await
        .map_err(map_register_err)?;
        Ok(())
    }

    /// Create new comment
    async fn new_comment(&self, comment: &CreateGistComment) -> DBResult<i64> {
        let now = now_unix_time_stamp();
        sqlx::query!(
            "INSERT INTO gists_comments (owner_id, gist_id, comment, created)
            VALUES (
                (SELECT ID FROM gists_users WHERE username = $1),
                (SELECT ID FROM gists_gists WHERE public_id = $2),
                $3,
                $4
            )",
            comment.owner,
            comment.gist_public_id,
            comment.comment,
            now,
        )
        .execute(&self.pool)
        .await
        .map_err(map_register_err)?;
        #[allow(non_snake_case)]
        struct ID {
            ID: i64,
        }

        let res = sqlx::query_as!(
            ID,
            "
            SELECT
                ID
            FROM
                gists_comments_view
            WHERE
                owner = $1
            AND
                gist_public_id = $2
            AND
                created = $3
            AND
                comment = $4
            ",
            comment.owner,
            comment.gist_public_id,
            now,
            comment.comment,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            Error::RowNotFound => DBError::CommentNotFound,
            e => DBError::DBError(Box::new(e)),
        })?;

        Ok(res.ID)
    }
    /// Get comments on a gist
    async fn get_comments_on_gist(&self, public_id: &str) -> DBResult<Vec<GistComment>> {
        let mut res = sqlx::query_as!(
            InnerGistComment,
            "
            SELECT
                ID,
                comment,
                owner,
                created,
                gist_public_id
            FROM
                gists_comments_view
            WHERE
                gist_public_id = $1
            ORDER BY created;
            ",
            public_id,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| match e {
            Error::RowNotFound => DBError::CommentNotFound,
            e => DBError::DBError(Box::new(e)),
        })?;

        let mut comments: Vec<GistComment> = Vec::with_capacity(res.len());
        res.drain(..).for_each(|r| comments.push(r.into()));
        Ok(comments)
    }
    /// Get a specific comment using its database assigned ID
    async fn get_comment_by_id(&self, id: i64) -> DBResult<GistComment> {
        let res = sqlx::query_as!(
            InnerGistComment,
            "
            SELECT
                ID,
                comment,
                owner,
                created,
                gist_public_id
            FROM
                gists_comments_view
            WHERE
                ID = $1
            ",
            id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            Error::RowNotFound => DBError::CommentNotFound,
            e => DBError::DBError(Box::new(e)),
        })?;

        Ok(res.into())
    }

    /// Delete comment
    async fn delete_comment(&self, owner: &str, id: i64) -> DBResult<()> {
        sqlx::query!(
            "DELETE FROM gists_comments
                    WHERE
                        ID = $1
                    AND
                        owner_id = (SELECT ID FROM gists_users WHERE username = $2)
                    ",
            id,
            owner,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DBError::DBError(Box::new(e)))?;
        Ok(())
    }

    async fn visibility_exists(&self, visibility: &GistVisibility) -> DBResult<bool> {
        let visibility = visibility.to_str();
        match sqlx::query!(
            "SELECT ID from gists_visibility WHERE name = $1",
            visibility
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(_) => Ok(true),
            Err(Error::RowNotFound) => Ok(false),
            Err(e) => Err(DBError::DBError(Box::new(e))),
        }
    }
}

fn now_unix_time_stamp() -> i64 {
    OffsetDateTime::now_utc().unix_timestamp()
}

struct InnerGist {
    owner: String,
    description: Option<String>,
    public_id: String,
    created: i64,
    updated: i64,
    visibility: String,
}

impl InnerGist {
    fn into_gist(self) -> DBResult<Gist> {
        Ok(Gist {
            owner: self.owner,
            description: self.description,
            public_id: self.public_id,
            created: self.created,
            updated: self.updated,
            visibility: GistVisibility::from_str(&self.visibility)?,
        })
    }
}

#[allow(non_snake_case)]
struct InnerGistComment {
    ID: i64,
    owner: String,
    comment: Option<String>,
    gist_public_id: String,
    created: i64,
}

impl From<InnerGistComment> for GistComment {
    fn from(g: InnerGistComment) -> Self {
        Self {
            id: g.ID,
            owner: g.owner,
            comment: g.comment.unwrap(),
            gist_public_id: g.gist_public_id,
            created: g.created,
        }
    }
}
