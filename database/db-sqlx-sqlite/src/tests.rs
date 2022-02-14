use sqlx::sqlite::SqlitePoolOptions;
use std::env;

use crate::*;

use db_core::tests::*;

#[actix_rt::test]
async fn everyting_works() {
    const EMAIL: &str = "sqliteuser@foo.com";
    const EMAIL2: &str = "sqliteuse2r@foo.com";
    const NAME: &str = "sqliteuser";
    const NAME2: &str = "sqliteuser2";
    const NAME3: &str = "sqliteuser3";
    const NAME4: &str = "sqliteuser4";
    const NAME5: &str = "sqliteuser5";
    const NAME6: &str = "sqliteuser6";
    const NAME7: &str = "sqliteuser7";
    const PASSWORD: &str = "pasdfasdfasdfadf";
    const SECRET1: &str = "sqlitesecret1";
    const SECRET2: &str = "sqlitesecret2";
    const SECRET3: &str = "sqlitesecret3";
    const SECRET4: &str = "sqlitesecret4";

    let url = env::var("SQLITE_DATABASE_URL").expect("Set SQLITE_DATABASE_URL env var");
    let pool_options = SqlitePoolOptions::new().max_connections(2);
    let connection_options = ConnectionOptions::Fresh(Fresh { pool_options, url });
    let db = connection_options.connect().await.unwrap();

    db.migrate().await.unwrap();
    email_register_works(&db, EMAIL, NAME, PASSWORD, SECRET1, NAME5).await;
    username_register_works(&db, NAME2, PASSWORD, SECRET2).await;
    duplicate_secret_guard_works(&db, NAME3, PASSWORD, NAME4, SECRET3, SECRET2).await;
    duplicate_username_and_email(&db, NAME6, NAME7, EMAIL2, PASSWORD, SECRET4, NAME, EMAIL).await;
    let creds = Creds {
        username: NAME.into(),
        password: SECRET4.into(),
    };
    db.update_password(&creds).await.unwrap();
}

#[actix_rt::test]
async fn visibility_test() {
    let url = env::var("SQLITE_DATABASE_URL").expect("Set SQLITE_DATABASE_URL env var");
    let pool_options = SqlitePoolOptions::new().max_connections(2);
    let connection_options = ConnectionOptions::Fresh(Fresh { pool_options, url });
    let db = connection_options.connect().await.unwrap();

    db.migrate().await.unwrap();
    visibility_works(&db).await;
}

#[actix_rt::test]
async fn gist_test() {
    const NAME: &str = "postgisttest";
    const PASSWORD: &str = "pasdfasdfasdfadf";
    const SECRET: &str = "postgisttestsecret";
    const PUBLIC_ID: &str = "postgisttestsecret";

    let url = env::var("SQLITE_DATABASE_URL").expect("Set SQLITE_DATABASE_URL env var");
    let pool_options = SqlitePoolOptions::new().max_connections(2);
    let connection_options = ConnectionOptions::Fresh(Fresh { pool_options, url });
    let db = connection_options.connect().await.unwrap();

    db.migrate().await.unwrap();
    gists_work(&db, NAME, PASSWORD, SECRET, PUBLIC_ID).await;
}
