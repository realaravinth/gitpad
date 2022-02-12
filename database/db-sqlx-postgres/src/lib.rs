#![deny(missing_docs)]
//! # `libadmin` database operations implemented using sqlx postgres
//!
//! [`GistDatabase`](GistDatabase) is implemented on [Database].

use db_core::dev::*;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

mod errors;
#[cfg(test)]
pub mod tests;

/// Database pool. All database functionallity(`libadmin` traits) are implemented on this
/// data structure
#[derive(Clone)]
pub struct Database {
    /// database pool
    pub pool: PgPool,
}

/// Use an existing database pool
pub struct Conn(pub PgPool);

/// Connect to databse
pub enum ConnectionOptions {
    /// fresh connection
    Fresh(Fresh),
    /// existing connection
    Existing(Conn),
}

/// Create a new database pool
pub struct Fresh {
    /// Pool options
    pub pool_options: PgPoolOptions,
    /// database URL
    pub url: String,
}

pub mod dev {
    //! useful imports for supporting a new database
    pub use super::errors::*;
    pub use super::Database;
    pub use db_core::dev::*;
    pub use prelude::*;
    pub use sqlx::Error;
}

pub mod prelude {
    //! useful imports for users working with a supported database
    pub use super::*;
    pub use db_core::prelude::*;
}
use dev::*;

#[async_trait]
impl Connect for ConnectionOptions {
    type Pool = Database;
    /// create connection pool
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
impl GistDatabase for Database {
    async fn email_login(&self, email: &str) -> DBResult<Creds> {
        sqlx::query_as!(
            Creds,
            r#"SELECT username, password  FROM admin_users WHERE email = ($1)"#,
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
            r#"SELECT password  FROM admin_users WHERE username = ($1)"#,
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
            "insert into admin_users 
        (username , password, email, secret) values ($1, $2, $3, $4)",
            &payload.username,
            &payload.password,
            &payload.email,
            &payload.secret,
        )
        .execute(&self.pool)
        .await
        .map_err(map_register_err)?;
        Ok(())
    }

    async fn username_register(&self, payload: &UsernameRegisterPayload) -> DBResult<()> {
        sqlx::query!(
            "INSERT INTO admin_users 
        (username , password,  secret) VALUES ($1, $2, $3)",
            &payload.username,
            &payload.password,
            &payload.secret,
        )
        .execute(&self.pool)
        .await
        .map_err(map_register_err)?;
        Ok(())
    }

    async fn update_email(&self, payload: &UpdateEmailPayload) -> DBResult<()> {
        let x = sqlx::query!(
            "UPDATE admin_users set email = $1
        WHERE username = $2",
            &payload.email,
            &payload.username,
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
            "UPDATE admin_users set password = $1
        WHERE username = $2",
            &payload.password,
            &payload.username,
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
        let res = sqlx::query!(
            "SELECT EXISTS (SELECT 1 from admin_users WHERE email = $1)",
            &email
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DBError::DBError(Box::new(e)))?;

        let mut exists = false;
        if let Some(x) = res.exists {
            if x {
                exists = true;
            }
        };

        Ok(exists)
    }

    async fn delete_account(&self, username: &str) -> DBResult<()> {
        sqlx::query!("DELETE FROM admin_users WHERE username = ($1)", username,)
            .execute(&self.pool)
            .await
            .map_err(|e| DBError::DBError(Box::new(e)))?;
        Ok(())
    }

    async fn username_exists(&self, username: &str) -> DBResult<bool> {
        let res = sqlx::query!(
            "SELECT EXISTS (SELECT 1 from admin_users WHERE username = $1)",
            &username
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DBError::DBError(Box::new(e)))?;

        let mut exists = false;
        if let Some(x) = res.exists {
            if x {
                exists = true;
            }
        };

        Ok(exists)
    }

    async fn update_username(&self, payload: &UpdateUsernamePayload) -> DBResult<()> {
        let x = sqlx::query!(
            "UPDATE admin_users set username = $1 WHERE username = $2",
            &payload.new_username,
            &payload.old_username,
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
            "UPDATE admin_users set secret = $1
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
            r#"SELECT secret  FROM admin_users WHERE username = ($1)"#,
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
}
