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
use std::{env, fs};

use config::{Config, ConfigError, Environment, File};
use derive_more::Display;
use log::warn;
use serde::Deserialize;
use url::Url;

#[derive(Debug, Clone, Deserialize)]
pub struct Server {
    pub port: u32,
    pub domain: String,
    pub ip: String,
    pub proxy_has_tls: bool,
    pub cookie_secret: String,
    pub workers: Option<usize>,
}

impl Server {
    #[cfg(not(tarpaulin_include))]
    pub fn get_ip(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}

#[derive(Deserialize, Display, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    #[display(fmt = "debug")]
    Debug,
    #[display(fmt = "info")]
    Info,
    #[display(fmt = "trace")]
    Trace,
    #[display(fmt = "error")]
    Error,
    #[display(fmt = "warn")]
    Warn,
}

impl LogLevel {
    fn set_log_level(&self) {
        const LOG_VAR: &str = "RUST_LOG";
        if env::var(LOG_VAR).is_err() {
            env::set_var("RUST_LOG", format!("{}", self));
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Repository {
    pub root: String,
}

impl Repository {
    fn create_root_dir(&self) {
        let root = Path::new(&self.root);
        if root.exists() {
            if !root.is_dir() {
                fs::remove_file(&root).unwrap();
                fs::create_dir_all(&root).unwrap();
            }
        } else {
            fs::create_dir_all(&root).unwrap();
        }
    }
}

#[derive(Deserialize, Display, PartialEq, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum DBType {
    #[display(fmt = "postgres")]
    Postgres,
    #[display(fmt = "sqlite")]
    Sqlite,
}

impl DBType {
    fn from_url(url: &Url) -> Result<Self, ConfigError> {
        match url.scheme() {
            "sqlite" => Ok(Self::Sqlite),
            "postgres" => Ok(Self::Postgres),
            _ => Err(ConfigError::Message("Unknown database type".into())),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct DatabaseBuilder {
    pub port: u32,
    pub hostname: String,
    pub username: String,
    pub password: String,
    pub name: String,
    pub database_type: DBType,
}

impl DatabaseBuilder {
    #[cfg(not(tarpaulin_include))]
    fn extract_database_url(url: &Url) -> Self {
        log::debug!("Databse name: {}", url.path());
        let mut path = url.path().split('/');
        path.next();
        let name = path.next().expect("no database name").to_string();
        DatabaseBuilder {
            port: url.port().expect("Enter database port").into(),
            hostname: url.host().expect("Enter database host").to_string(),
            username: url.username().into(),
            password: url.password().expect("Enter database password").into(),
            name,
            database_type: DBType::from_url(url).unwrap(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Database {
    pub url: String,
    pub pool: u32,
    pub database_type: DBType,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub log: LogLevel,
    pub database: Database,
    pub allow_registration: bool,
    pub allow_demo: bool,
    pub server: Server,
    pub source_code: String,
    pub repository: Repository,
}

#[cfg(not(tarpaulin_include))]
impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();

        // setting default values
        #[cfg(test)]
        s.set_default("database.pool", 2.to_string())
            .expect("Couldn't get the number of CPUs");

        const CURRENT_DIR: &str = "./config/default.toml";
        const ETC: &str = "/etc/gists/config.toml";

        if let Ok(path) = env::var("GIST_CONFIG") {
            s.merge(File::with_name(&path))?;
        } else if Path::new(CURRENT_DIR).exists() {
            // merging default config from file
            s.merge(File::with_name(CURRENT_DIR))?;
        } else if Path::new(ETC).exists() {
            s.merge(File::with_name(ETC))?;
        } else {
            log::warn!("configuration file not found");
        }

        s.merge(Environment::with_prefix("GISTS").separator("__"))?;

        check_url(&s);

        match env::var("PORT") {
            Ok(val) => {
                s.set("server.port", val).unwrap();
            }
            Err(e) => warn!("couldn't interpret PORT: {}", e),
        }

        match env::var("DATABASE_URL") {
            Ok(val) => {
                let url = Url::parse(&val).expect("couldn't parse Database URL");
                let database_conf = DatabaseBuilder::extract_database_url(&url);
                set_from_database_url(&mut s, &database_conf);
            }
            Err(e) => warn!("couldn't interpret DATABASE_URL: {}", e),
        }

        set_database_url(&mut s);

        let settings: Settings = s.try_into()?;

        settings.log.set_log_level();
        settings.repository.create_root_dir();

        Ok(settings)
    }
}

#[cfg(not(tarpaulin_include))]
fn check_url(s: &Config) {
    let url = s
        .get::<String>("source_code")
        .expect("Couldn't access source_code");

    Url::parse(&url).expect("Please enter a URL for source_code in settings");
}

#[cfg(not(tarpaulin_include))]
fn set_from_database_url(s: &mut Config, database_conf: &DatabaseBuilder) {
    s.set("database.username", database_conf.username.clone())
        .expect("Couldn't set database username");
    s.set("database.password", database_conf.password.clone())
        .expect("Couldn't access database password");
    s.set("database.hostname", database_conf.hostname.clone())
        .expect("Couldn't access database hostname");
    s.set("database.port", database_conf.port as i64)
        .expect("Couldn't access database port");
    s.set("database.name", database_conf.name.clone())
        .expect("Couldn't access database name");
    s.set(
        "database.database_type",
        format!("{}", database_conf.database_type),
    )
    .expect("Couldn't access database type");
}

#[cfg(not(tarpaulin_include))]
fn set_database_url(s: &mut Config) {
    s.set(
        "database.url",
        format!(
            r"{}://{}:{}@{}:{}/{}",
            s.get::<String>("database.database_type")
                .expect("Couldn't access database database_type"),
            s.get::<String>("database.username")
                .expect("Couldn't access database username"),
            s.get::<String>("database.password")
                .expect("Couldn't access database password"),
            s.get::<String>("database.hostname")
                .expect("Couldn't access database hostname"),
            s.get::<String>("database.port")
                .expect("Couldn't access database port"),
            s.get::<String>("database.name")
                .expect("Couldn't access database name")
        ),
    )
    .expect("Couldn't set databse url");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::get_random;

    #[test]
    fn database_type_test() {
        for i in ["sqlite://foo", "postgres://bar", "unknown://"].iter() {
            let url = Url::parse(i).unwrap();
            if i.contains("sqlite") {
                assert_eq!(DBType::from_url(&url).unwrap(), DBType::Sqlite);
            } else if i.contains("unknown") {
                assert!(DBType::from_url(&url).is_err());
            } else {
                assert_eq!(DBType::from_url(&url).unwrap(), DBType::Postgres);
            }
        }
    }

    #[test]
    fn root_dir_is_created_test() {
        let dir;
        loop {
            let mut tmp = env::temp_dir();
            tmp = tmp.join(get_random(10));

            if tmp.exists() {
                continue;
            } else {
                dir = tmp;
                break;
            }
        }

        let repo = Repository {
            root: dir.to_str().unwrap().to_owned(),
        };

        repo.create_root_dir();
        assert!(dir.exists());
        assert!(dir.is_dir());
        let file = dir.join("foo");
        fs::write(&file, "foo").unwrap();
        repo.create_root_dir();
        assert!(dir.exists());
        assert!(dir.is_dir());

        assert!(file.exists());
        assert!(file.is_file());

        let repo = Repository {
            root: file.to_str().unwrap().to_owned(),
        };

        repo.create_root_dir();
        assert!(file.exists());
        assert!(file.is_dir());
    }
}
