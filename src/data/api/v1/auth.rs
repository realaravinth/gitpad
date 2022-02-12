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
//! Authentication helper methods and data structures
use db_core::prelude::*;
use serde::{Deserialize, Serialize};

use super::get_random;
use crate::errors::*;
use crate::Data;

/// Register payload
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Register {
    /// username
    pub username: String,
    /// password
    pub password: String,
    /// password confirmation: `password` and `confirm_password` must match
    pub confirm_password: String,
    /// optional email
    pub email: Option<String>,
}

/// Login payload
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Login {
    // login accepts both username and email under "username field"
    // TODO update all instances where login is used
    /// user identifier: either username or email
    /// an email is detected by checkinf for the existence of `@` character
    pub login: String,
    /// password
    pub password: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
/// struct used to represent password
pub struct Password {
    /// password
    pub password: String,
}

impl Data {
    /// Log in method. Returns `Ok(())` when user is authenticated and errors when authentication
    /// fails
    pub async fn login<T: GistDatabase>(&self, db: &T, payload: &Login) -> ServiceResult<String> {
        use argon2_creds::Config;

        let verify = |stored: &str, received: &str| {
            if Config::verify(stored, received)? {
                Ok(())
            } else {
                Err(ServiceError::WrongPassword)
            }
        };

        if payload.login.contains('@') {
            let creds = db.email_login(&payload.login).await?;
            verify(&creds.password, &payload.password)?;
            Ok(creds.username)
        } else {
            let password = db.username_login(&payload.login).await?;
            verify(&password.password, &payload.password)?;
            Ok(payload.login.clone())
        }
    }

    /// register new user
    pub async fn register<T: GistDatabase>(&self, db: &T, payload: &Register) -> ServiceResult<()> {
        if !self.settings.allow_registration {
            return Err(ServiceError::ClosedForRegistration);
        }

        if payload.password != payload.confirm_password {
            return Err(ServiceError::PasswordsDontMatch);
        }
        let username = self.creds.username(&payload.username)?;
        let hash = self.creds.password(&payload.password)?;

        if let Some(email) = &payload.email {
            self.creds.email(email)?;
        }

        let mut secret;

        if let Some(email) = &payload.email {
            loop {
                secret = get_random(32);

                let db_payload = EmailRegisterPayload {
                    secret: &secret,
                    username: &username,
                    password: &hash,
                    email,
                };

                match db.email_register(&db_payload).await {
                    Ok(_) => break,
                    Err(DBError::DuplicateSecret) => continue,
                    Err(e) => return Err(e.into()),
                }
            }
        } else {
            loop {
                secret = get_random(32);

                let db_payload = UsernameRegisterPayload {
                    secret: &secret,
                    username: &username,
                    password: &hash,
                };

                match db.username_register(&db_payload).await {
                    Ok(_) => break,
                    Err(DBError::DuplicateSecret) => continue,
                    Err(e) => return Err(e.into()),
                }
            }
        }
        Ok(())
    }
}
